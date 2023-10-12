use std::collections::HashMap;

use rand::Rng;
use serde::Deserialize;

use crate::{scenario::{StepData, StepResult, StepOutcome}, utils::value::Value};

use super::{TaskParam, Task};

#[derive(Clone, Debug)]
pub struct InitAccount {}

impl Task for InitAccount {
    type P = InitAccountParameters;

    fn run(dto: <<Self as Task>::P as TaskParam>::D, state: &HashMap<u64, StepData>) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        println!("namadac init-account --keys {:?} --alias test --threshold {}", parameters.keys, parameters.threshold);

        let mut data = HashMap::new();
        data.insert("address_alias".to_string(), "alias".to_string());

        StepResult { outcome: StepOutcome::successful(), data: StepData { data } }
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

    fn from_dto(dto: Self::D, state: &HashMap<u64, StepData>) -> Self {
        let keys = dto
            .keys
            .iter()
            .map(|value: &Value| match value {
                Value::Ref { value } => state
                    .get(value)
                    .expect("id should be valid")
                    .get_field("key-alias"),
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
                Value::Fuzz {} => rand::thread_rng().gen_range(1..keys.len()) as u64,
            },
            None => 1u64,
        };

        Self { keys, threshold }
    }
}
