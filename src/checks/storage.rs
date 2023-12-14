use async_trait::async_trait;
use serde::Deserialize;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct StorageCheck {}

impl StorageCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for StorageCheck {
    type P = StorageCheckParameters;

    async fn execute(&self, _sdk: &Sdk, parameters: Self::P, state: &Storage) -> StepResult {
        let data = state.get_step_item(&parameters.step, &parameters.field);

        if data.eq(&parameters.value) {
            StepResult::success_empty()
        } else {
            StepResult::fail()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageCheckParametersDto {
    step: u64,
    field: String,
    value: Value,
}

#[derive(Clone, Debug)]
pub struct StorageCheckParameters {
    step: u64,
    field: String,
    value: String,
}

impl CheckParam for StorageCheckParameters {
    type D = StorageCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let step = dto.step;
        let field = dto.field;
        let value = match dto.value {
            Value::Ref { value, field } => state.get_step_item(&value, &field),
            Value::Value { value } => value,
            Value::Fuzz {} => unimplemented!(),
        };

        Self { step, field, value }
    }
}
