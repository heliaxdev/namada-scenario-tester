use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
    utils::value::Value,
};
use namada_sdk::Namada;
use namada_sdk::{args::TxBuilder, signing::default_sign};

use super::{Task, TaskParam};

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
    pub fn generate_random_alias(&self) -> String {
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        format!("lt-acc-{}", random_suffix)
    }
}

#[async_trait(?Send)]
impl Task for TxInitAccount {
    type P = TxInitAccountParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let random_alias = self.generate_random_alias();

        let wallet = sdk.namada.wallet.read().await;
        let mut public_keys = vec![];
        for source in parameters.sources {
            let pk = source.to_public_key(sdk).await;
            public_keys.push(pk);
        }

        drop(wallet);

        let init_account_tx_builder = sdk
            .namada
            .new_init_account(public_keys.clone(), Some(parameters.threshold as u8))
            .initialized_account_alias(random_alias.clone())
            .wallet_alias_force(true)
            .signing_keys(public_keys.clone());

        let (mut init_account_tx, signing_data) = init_account_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

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
        let tx_submission = sdk
            .namada
            .submit(init_account_tx, &init_account_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();

        let account_address = match tx_submission {
            Ok(process_tx_response) => match process_tx_response {
                namada_sdk::tx::ProcessTxResponse::Applied(tx_response) => {
                    if let Some(mut tx_inner) = tx_response.inner_tx {
                        tx_inner.initialized_accounts.pop().unwrap()
                    } else {
                        self.fetch_info(sdk, &mut storage).await;
                        return StepResult::fail();
                    }
                }
                namada_sdk::tx::ProcessTxResponse::Broadcast(_) => {
                    self.fetch_info(sdk, &mut storage).await;
                    return StepResult::fail();
                }
                namada_sdk::tx::ProcessTxResponse::DryRun(mut tx_result) => {
                    tx_result.initialized_accounts.pop().unwrap()
                }
            },
            Err(_) => {
                self.fetch_info(sdk, &mut storage).await;
                return StepResult::fail();
            }
        };

        storage.add(
            TxInitAccountStorageKeys::Alias.to_string(),
            random_alias.to_string(),
        );
        storage.add(
            TxInitAccountStorageKeys::Address.to_string(),
            account_address.to_string(),
        );
        storage.add(
            TxInitAccountStorageKeys::Threshold.to_string(),
            parameters.threshold.to_string(),
        );
        storage.add(
            TxInitAccountStorageKeys::TotalPublicKeys.to_string(),
            public_keys.len().to_string(),
        );
        for (key, value) in public_keys.clone().into_iter().enumerate() {
            storage.add(
                TxInitAccountStorageKeys::PublicKeyAtIndex(key as u8).to_string(),
                value.to_string(),
            );
        }

        self.fetch_info(sdk, &mut storage).await;

        let account = StateAddress::new_enstablished(
            random_alias,
            account_address.to_string(),
            public_keys
                .iter()
                .map(|pk| pk.to_string())
                .collect::<Vec<String>>(),
            parameters.threshold,
        );

        StepResult::success_with_accounts(storage, vec![account])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxInitAccountParametersDto {
    sources: Vec<Value>,
    threshold: Option<Value>,
}

impl TxInitAccountParametersDto {
    pub fn new(sources: Vec<Value>, threshold: Option<Value>) -> Self {
        Self { sources, threshold }
    }
}

#[derive(Clone, Debug)]
pub struct TxInitAccountParameters {
    sources: Vec<AccountIndentifier>,
    threshold: u64,
}

impl TaskParam for TxInitAccountParameters {
    type D = TxInitAccountParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
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
                Value::Fuzz { value: _ } => unimplemented!(),
            })
            .collect::<Vec<AccountIndentifier>>();
        let threshold = match dto.threshold {
            Some(value) => match value {
                Value::Ref { .. } => unimplemented!(),
                Value::Value { value } => value
                    .parse::<u64>()
                    .expect("Should be convertiable to u64."),
                Value::Fuzz { value: _ } => rand::thread_rng().gen_range(1..=sources.len()) as u64,
            },
            None => 1u64,
        };

        Self { sources, threshold }
    }
}
