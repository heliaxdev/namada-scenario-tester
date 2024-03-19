use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    checks::bonds::BondsCheckParametersDto, scenario::StepType, utils::value::Value,
};

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckBond {
    amount: u64,
    delegate: u64, // step index
    delegator: Alias,
}

impl CheckBond {
    pub fn new(delegator: Alias, delegate: u64, amount: u64) -> Self {
        Self {
            amount,
            delegate,
            delegator,
        }
    }
}

impl Hook for CheckBond {
    fn to_json(&self) -> StepType {
        StepType::CheckBonds {
            parameters: BondsCheckParametersDto {
                amount: Value::v(self.amount.to_string()),
                delegate: Value::r(self.delegate, "validator-1-address".to_string()),
                delegator: Value::v(self.delegator.to_string()),
            },
        }
    }
}

impl Display for CheckBond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check bond from delegator {}", self.delegator)
    }
}
