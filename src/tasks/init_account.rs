use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct InitAccount {
    rpc: String,
    chain_id: String,
}

impl InitAccount {
    pub fn new(rpc: String, chain_id: String) -> Self {
        Self { rpc, chain_id }
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

impl Task for InitAccount {
    type P = InitAccountParameters;

    fn execute(&self, parameters: Self::P, _state: &Storage) -> StepResult {
        let alias = self.generate_random_alias();
        println!(
            "namadac init-account --keys {:?} --alias {} --threshold {} --node {}",
            parameters.keys, alias, parameters.threshold, format!("{}/{}", self.rpc, self.chain_id)
        );

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), alias.to_string());
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

        let account = Address::new(
            alias,
            "todo".to_string(),
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
