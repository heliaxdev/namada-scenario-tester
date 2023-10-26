use async_trait::async_trait;
use serde::Deserialize;

use crate::{scenario::StepResult, state::state::Storage, utils::value::Value, sdk::namada::Sdk};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct TxCheck {
    rpc: String,
    chain_id: String,
}

impl TxCheck {
    pub fn new(sdk: &Sdk) -> Self {
        Self {
            rpc: sdk.rpc.clone(),
            chain_id: sdk.chain_id.clone(),
        }
    }
}

#[async_trait(?Send)]
impl Check for TxCheck {
    type P = TxCheckParameters;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult {
        let step_outcome = state.is_step_successful(&paramaters.id);

        if step_outcome.eq(&paramaters.outcome) {
            StepResult::success_empty()
        } else {
            StepResult::fail()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxCheckParametersDto {
    outcome: Value,
    id: Value,
}

#[derive(Clone, Debug)]
pub struct TxCheckParameters {
    outcome: bool,
    id: u64,
}

impl CheckParam for TxCheckParameters {
    type D = TxCheckParametersDto;

    fn from_dto(dto: Self::D, _state: &Storage) -> Self {
        let outcome = match dto.outcome {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.eq("success"),
            Value::Fuzz {} => unimplemented!(),
        };
        let id = match dto.id {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self { outcome, id }
    }
}
