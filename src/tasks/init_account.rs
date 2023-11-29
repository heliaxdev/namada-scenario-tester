use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
    utils::value::Value,
};
use namada_sdk::{args::TxBuilder, signing::default_sign};
use namada_sdk::{
    core::types::key::{
        common::{PublicKey, SecretKey},
        RefTo,
    },
    Namada,
};

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
        let alias = self.generate_random_alias();

        let mut wallet = sdk.namada.wallet.write().await;
        let secret_keys = parameters
            .keys
            .iter()
            .filter_map(|alias| wallet.find_secret_key(alias, None).ok())
            .collect::<Vec<SecretKey>>();
        let public_keys = secret_keys
            .iter()
            .map(|sk| sk.ref_to())
            .collect::<Vec<PublicKey>>();

        let init_account_tx_builder = sdk
            .namada
            .new_init_account(public_keys.clone(), Some(parameters.threshold as u8))
            .signing_keys(secret_keys);
        let (mut init_account_tx, signing_data, _epoch) = init_account_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");
        sdk.namada
            .sign(
                &mut init_account_tx,
                &init_account_tx_builder.tx,
                signing_data,
                default_sign,
            )
            .await
            .expect("unable to sign tx");
        let tx_submission = sdk
            .namada
            .submit(init_account_tx, &init_account_tx_builder.tx)
            .await;

        let account_address = if let Ok(tx_result) = tx_submission {
            tx_result.initialized_accounts().pop().unwrap()
        } else {
            return StepResult::fail();
        };

        let mut storage = StepStorage::default();
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
        for (key, value) in public_keys.into_iter().enumerate() {
            storage.add(
                TxInitAccountStorageKeys::PublicKeyAtIndex(key as u8).to_string(),
                value.to_string(),
            );
        }

        self.fetch_info(sdk, &mut storage).await;

        let account = StateAddress::new_enstablished(
            alias,
            account_address.to_string(),
            parameters.keys,
            parameters.threshold,
        );

        StepResult::success_with_accounts(storage, vec![account])
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxInitAccountParametersDto {
    keys: Vec<Value>,
    threshold: Option<Value>,
}

impl TxInitAccountParametersDto {
    pub fn new(keys: Vec<Value>, threshold: Option<Value>) -> Self {
        Self { keys, threshold }
    }
}

#[derive(Clone, Debug)]
pub struct TxInitAccountParameters {
    keys: Vec<String>,
    threshold: u64,
}

impl TaskParam for TxInitAccountParameters {
    type D = TxInitAccountParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let keys = dto
            .keys
            .iter()
            .map(|value: &Value| match value {
                Value::Ref { value } => state.get_step_item(value, "address-alias"),
                Value::Value { value } => value.to_owned(),
                Value::Fuzz {} => unimplemented!(),
            })
            .collect::<Vec<String>>();
        let threshold = match dto.threshold {
            Some(value) => match value {
                Value::Ref { .. } => unimplemented!(),
                Value::Value { value } => value
                    .parse::<u64>()
                    .expect("Should be convertiable to u64."),
                Value::Fuzz {} => rand::thread_rng().gen_range(1..=keys.len()) as u64,
            },
            None => 1u64,
        };

        Self { keys, threshold }
    }
}
