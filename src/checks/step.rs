use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct StepCheck {}

impl StepCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for StepCheck {
    type P = StepCheckParameters;

    async fn execute(&self, _sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult {
        let step_outcome =
            state.is_step_successful(&paramaters.id) || state.is_step_noop(&paramaters.id);

        if step_outcome.eq(&paramaters.outcome) {
            StepResult::success_empty()
        } else {
            StepResult::fail_check(step_outcome.to_string(), paramaters.outcome.to_string())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StepCheckParametersDto {
    pub outcome: Value,
    pub id: Value,
}

#[derive(Clone, Debug)]
pub struct StepCheckParameters {
    outcome: bool,
    id: u64,
}

impl CheckParam for StepCheckParameters {
    type D = StepCheckParametersDto;

    fn from_dto(dto: Self::D, _state: &Storage) -> Option<Self> {
        let outcome = match dto.outcome {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.eq("success"),
            Value::Fuzz { .. } => unimplemented!(),
        };
        let id = match dto.id {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Some(Self { outcome, id })
    }
}
