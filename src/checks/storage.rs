use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
            StepResult::fail_check(data, parameters.value)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    fn from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let step = dto.step;
        let field = dto.field;
        let value = match dto.value {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                state.get_step_item(&value, &field)
            }
            Value::Value { value } => value,
            Value::Fuzz { .. } => unimplemented!(),
        };

        Some(Self { step, field, value })
    }
}
