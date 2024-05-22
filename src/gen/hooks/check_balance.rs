use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    checks::balance::BalanceCheckParametersDto, queries::balance::BalanceQueryStorageKeys,
    scenario::StepType, utils::value::Value,
};

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckBalance {
    owner: Alias, // the step id from a QueryBalance step
    token: Alias,
    amount_step_id: u64,
    op: String,
}

impl CheckBalance {
    pub fn new(step_id: u64, owner: Alias, token: Alias, op: String) -> Self {
        Self {
            owner,
            token,
            amount_step_id: step_id,
            op,
        }
    }
}

impl Hook for CheckBalance {
    fn to_step_type(&self) -> StepType {
        StepType::CheckBalance {
            parameters: BalanceCheckParametersDto {
                amount: Value::r(
                    self.amount_step_id,
                    BalanceQueryStorageKeys::Amount.to_string(),
                ),
                address: Value::v(self.owner.to_string()),
                token: Value::v(self.token.to_string()),
                op: Value::v(self.op.to_string()),
            },
        }
    }
}

impl Display for CheckBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check balance at step id {}", self.amount_step_id)
    }
}
