use std::fmt::Display;

use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};
use namada_sdk::{args::TxBuilder, Namada};
use namada_sdk::{args::TxInitAccount as SdkInitAccountTx, signing::default_sign};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxInitAccountStorageKeys {
    Alias,
    Address,
    Threshold,
    TotalPublicKeys,
    PublicKeyAtIndex(u8),
}

impl ToString for TxInitAccountStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitAccountStorageKeys::Alias => "alias".to_string(),
            TxInitAccountStorageKeys::Address => "address".to_string(),
            TxInitAccountStorageKeys::Threshold => "threshold".to_string(),
            TxInitAccountStorageKeys::TotalPublicKeys => "total_public_keys".to_string(),
            TxInitAccountStorageKeys::PublicKeyAtIndex(index) => {
                format!("public_key_at_index-{}", index)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxInitAccount {}

impl TxInitAccount {
    pub fn new() -> Self {
        Self {}
    }
}

impl TxInitAccount {
    pub fn generate_random_alias() -> String {
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        format!("lt-acc-enst-{}", random_suffix)
    }
}

#[async_trait(?Send)]
impl Task for TxInitAccount {
    type P = TxInitAccountParameters;
    type B = SdkInitAccountTx;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let alias = parameters.alias;

        let mut public_keys = vec![];
        for source in parameters.sources {
            let pk = source.to_public_key(sdk).await;
            public_keys.push(pk);
        }

        let init_account_tx_builder = sdk
            .namada
            .new_init_account(public_keys.clone(), Some(parameters.threshold as u8))
            .initialized_account_alias(alias.clone())
            .wallet_alias_force(true);

        let init_account_tx_builder = self
            .add_settings(sdk, init_account_tx_builder, settings)
            .await;

        let (mut init_account_tx, signing_data) = init_account_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut init_account_tx,
                &init_account_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxInitAccountStorageKeys::Alias.to_string(),
            alias.to_string(),
        );
        step_storage.add(
            TxInitAccountStorageKeys::Threshold.to_string(),
            parameters.threshold.to_string(),
        );
        step_storage.add(
            TxInitAccountStorageKeys::TotalPublicKeys.to_string(),
            public_keys.len().to_string(),
        );
        for (key, value) in public_keys.clone().into_iter().enumerate() {
            step_storage.add(
                TxInitAccountStorageKeys::PublicKeyAtIndex(key as u8).to_string(),
                value.to_string(),
            );
        }

        Ok(BuildResult::new(
            init_account_tx,
            init_account_tx_builder.tx,
            step_storage,
        ))
    }

    async fn execute(
        &self,
        sdk: &Sdk,
        data: BuildResult,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let (tx, tx_args, mut step_storage) = if data.should_execute() {
            data.execute_data()
        } else {
            return Ok(StepResult::success(data.step_storage));
        };

        let tx_submission = sdk.namada.submit(tx.clone(), &tx_args).await;

        let cmt = tx.first_commitments().unwrap().to_owned();
        let wrapper_hash = tx.wrapper_hash();

        let account_address = match tx_submission {
            Ok(process_tx_response) => {
                match process_tx_response.is_applied_and_valid(wrapper_hash.as_ref(), &cmt) {
                    Some(tx_result) => {
                        if let Some(account) = tx_result.initialized_accounts.first() {
                            account.clone()
                        } else {
                            let log =
                                Self::get_tx_errors(&tx, &process_tx_response).unwrap_or_default();
                            return Ok(StepResult::fail(log));
                        }
                    }
                    None => {
                        let log =
                            Self::get_tx_errors(&tx, &process_tx_response).unwrap_or_default();
                        return Ok(StepResult::fail(log));
                    }
                }
            }
            Err(e) => {
                return Ok(StepResult::fail(e.to_string()));
            }
        };

        step_storage.add(
            TxInitAccountStorageKeys::Address.to_string(),
            account_address.to_string(),
        );

        Ok(StepResult::success_with_accounts(step_storage, vec![]))
    }
}

impl Display for TxInitAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-init-account")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxInitAccountParametersDto {
    pub alias: Value,
    pub sources: Vec<Value>,
    pub threshold: Option<Value>,
}

impl TxInitAccountParametersDto {
    pub fn new(alias: Value, sources: Vec<Value>, threshold: Option<Value>) -> Self {
        Self {
            alias,
            sources,
            threshold,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TxInitAccountParameters {
    alias: String,
    sources: Vec<AccountIndentifier>,
    threshold: u64,
}

impl TaskParam for TxInitAccountParameters {
    type D = TxInitAccountParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let alias = match dto.alias {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.to_string(),
            Value::Fuzz { .. } => TxInitAccount::generate_random_alias(),
        };
        let sources = dto
            .sources
            .into_iter()
            .map(|value| match value {
                Value::Ref { value, field } => {
                    let data = state.get_step_item(&value, &field);
                    match field.to_lowercase().as_str() {
                        "alias" => AccountIndentifier::Alias(data),
                        "public-key" => AccountIndentifier::PublicKey(data),
                        "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                        _ => AccountIndentifier::Address(data),
                    }
                }
                Value::Value { value } => {
                    if value.starts_with(ADDRESS_PREFIX) {
                        AccountIndentifier::Address(value)
                    } else {
                        AccountIndentifier::Alias(value)
                    }
                }
                Value::Fuzz { .. } => unimplemented!(),
            })
            .collect::<Vec<AccountIndentifier>>();

        let threshold = match dto.threshold {
            Some(value) => match value {
                Value::Ref { .. } => unimplemented!(),
                Value::Value { value } => value
                    .parse::<u64>()
                    .expect("Should be convertiable to u64."),
                Value::Fuzz { .. } => rand::thread_rng().gen_range(1..=sources.len()) as u64,
            },
            None => 1u64,
        };

        Some(Self {
            alias,
            sources,
            threshold,
        })
    }
}
