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
use namada_sdk::Namada;
use namada_sdk::{
    args::{TxBuilder, TxUpdateAccount as SdkUpdateAccountTx},
    signing::default_sign,
};

use super::{Task, TaskParam};

pub enum TxUpdateAccountStorageKeys {
    Alias,
    Address,
    Threshold,
    TotalPublicKeys,
    PublicKeyAtIndex(u8),
}

impl ToString for TxUpdateAccountStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxUpdateAccountStorageKeys::Alias => "alias".to_string(),
            TxUpdateAccountStorageKeys::Address => "address".to_string(),
            TxUpdateAccountStorageKeys::Threshold => "threshold".to_string(),
            TxUpdateAccountStorageKeys::TotalPublicKeys => "total_public_keys".to_string(),
            TxUpdateAccountStorageKeys::PublicKeyAtIndex(index) => {
                format!("public_key_at_index-{}", index)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxUpdateAccount {}

impl TxUpdateAccount {
    pub fn new() -> Self {
        Self {}
    }
}

impl TxUpdateAccount {
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
impl Task for TxUpdateAccount {
    type P = TxUpdateAccountParameters;
    type B = SdkUpdateAccountTx;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let threshold = parameters.threshold as u8;

        let mut public_keys = vec![];
        for source in parameters.sources {
            let pk = source.to_public_key(sdk).await;
            public_keys.push(pk);
        }

        let update_account_tx_builder = sdk
            .namada
            .new_update_account(source_address)
            .public_keys(public_keys.clone())
            .threshold(threshold)
            .wallet_alias_force(true);

        let update_account_tx_builder = self
            .add_settings(sdk, update_account_tx_builder, settings)
            .await;

        let (mut update_account_tx, signing_data) = update_account_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

        sdk.namada
            .sign(
                &mut update_account_tx,
                &update_account_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");
        let tx_submission = sdk
            .namada
            .submit(update_account_tx, &update_account_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        let account_address = match tx_submission {
            Ok(process_tx_response) => match process_tx_response.is_applied_and_valid() {
                Some(tx_result) => {
                    if let Some(account) = tx_result.initialized_accounts.first() {
                        account.clone()
                    } else {
                        let log = Self::get_tx_errors(&process_tx_response).unwrap_or_default();
                        return StepResult::fail(log);
                    }
                }
                None => {
                    let log = Self::get_tx_errors(&process_tx_response).unwrap_or_default();
                    return StepResult::fail(log);
                }
            },
            Err(_e) => {
                return StepResult::fail("error sending tx".to_string());
            }
        };

        storage.add(
            TxUpdateAccountStorageKeys::Address.to_string(),
            account_address.to_string(),
        );
        storage.add(
            TxUpdateAccountStorageKeys::Threshold.to_string(),
            parameters.threshold.to_string(),
        );
        storage.add(
            TxUpdateAccountStorageKeys::TotalPublicKeys.to_string(),
            public_keys.len().to_string(),
        );
        for (key, value) in public_keys.clone().into_iter().enumerate() {
            storage.add(
                TxUpdateAccountStorageKeys::PublicKeyAtIndex(key as u8).to_string(),
                value.to_string(),
            );
        }

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxUpdateAccountParametersDto {
    pub source: Value,
    pub public_keys: Vec<Value>,
    pub threshold: Option<Value>,
}

impl TxUpdateAccountParametersDto {
    pub fn new(source: Value, public_keys: Vec<Value>, threshold: Option<Value>) -> Self {
        Self {
            source,
            public_keys,
            threshold,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TxUpdateAccountParameters {
    source: AccountIndentifier,
    sources: Vec<AccountIndentifier>,
    threshold: u64,
}

impl TaskParam for TxUpdateAccountParameters {
    type D = TxUpdateAccountParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
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
        };
        let sources = dto
            .public_keys
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

        Self {
            source,
            sources,
            threshold,
        }
    }
}