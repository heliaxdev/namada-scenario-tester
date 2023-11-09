use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng, random};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value,
};
use namada_sdk::{
    core::types::key::{common::{PublicKey, SecretKey}, RefTo},
    Namada,
};
use namada_sdk::{args::TxBuilder};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct InitAccount {}

impl InitAccount {
    pub fn new() -> Self {
        Self { }
    }
}

impl InitAccount {
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
impl Task for InitAccount {
    type P = InitAccountParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let alias = self.generate_random_alias();

        let mut wallet = sdk.namada.wallet.write().await;
        let secret_keys = parameters
            .keys
            .iter()
            .filter_map(|alias| wallet.find_key(alias, None).ok())
            .collect::<Vec<SecretKey>>();

        let public_keys = secret_keys
            .iter()
            .map(|sk| sk.ref_to())
            .collect::<Vec<PublicKey>>();
        drop(wallet);
        let random_alias = self.generate_random_alias();
        let init_account_tx_builder = sdk
            .namada
            .new_init_account(public_keys, Some(parameters.threshold as u8))
            .initialized_account_alias(random_alias)
            .signing_keys(secret_keys);
        let (mut init_account_tx, signing_data, _epoch) = init_account_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");
        // Gets stuck on signing init account tx
        sdk.namada
            .sign(
                &mut init_account_tx,
                &init_account_tx_builder.tx,
                signing_data,
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
            return StepResult::fail()
        };

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), alias.to_string());
        storage.add("address".to_string(), account_address.to_string());
        storage.add(
            "address-threshold".to_string(),
            parameters.threshold.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        let account = Address::new(
            alias,
            account_address.to_string(),
            parameters.keys,
            parameters.threshold,
        );

        StepResult::success_with_accounts(storage, vec![account])
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct InitAccountParametersDto {
    keys: Vec<Value>,
    threshold: Option<Value>,
}

impl InitAccountParametersDto {
    pub fn new(keys: Vec<Value>, threshold: Option<Value>) -> Self {
        Self { keys, threshold }
    }
}

#[derive(Clone, Debug)]
pub struct InitAccountParameters {
    keys: Vec<String>,
    threshold: u64,
}

impl TaskParam for InitAccountParameters {
    type D = InitAccountParametersDto;

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
