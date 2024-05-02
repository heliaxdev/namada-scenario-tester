use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    queries::balance::BalanceQueryParametersDto, scenario::StepType, utils::value::Value,
};

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct QueryBalance {
    owner: Alias,
    token: Alias,
}

impl QueryBalance {
    pub fn new(owner: Alias, token: Alias) -> Self {
        Self { owner, token }
    }
}

impl Hook for QueryBalance {
    fn to_step_type(&self) -> StepType {
        StepType::QueryAccountTokenBalance {
            parameters: BalanceQueryParametersDto {
                address: Value::v(self.owner.to_string()),
                token: Value::v(self.token.to_string()),
            },
        }
    }
}

impl Display for QueryBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query {} balance for token {}", self.owner, self.owner)
    }
}
