use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    checks::balance::BalanceCheckParametersDto, scenario::StepType, utils::value::Value,
};

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckBalance {
    alias: Alias,
    token: Alias,
    amount: u64,
}

impl CheckBalance {
    pub fn new(alias: Alias, token: Alias, amount: u64) -> Self {
        Self {
            alias,
            token,
            amount,
        }
    }
}

impl Hook for CheckBalance {
    fn to_step_type(&self) -> StepType {
        StepType::CheckBalance {
            parameters: BalanceCheckParametersDto {
                amount: Value::v(self.amount.to_string()),
                address: Value::v(self.alias.to_string()),
                token: Value::v(self.token.to_string()),
            },
        }
    }
}

impl Display for CheckBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check {} balance for {}", self.token, self.alias)
    }
}
