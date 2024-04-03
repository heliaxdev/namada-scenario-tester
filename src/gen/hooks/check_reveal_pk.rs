use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    checks::reveal_pk::RevealPkCheckParametersDto, scenario::StepType, utils::value::Value,
};

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckRevealPk {
    source: Alias,
}

impl CheckRevealPk {
    pub fn new(source: Alias) -> Self {
        Self { source }
    }
}

impl Hook for CheckRevealPk {
    fn to_step_type(&self) -> StepType {
        StepType::CheckRevealPk {
            parameters: RevealPkCheckParametersDto {
                source: Value::v(self.source.to_string()),
            },
        }
    }
}

impl Display for CheckRevealPk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check pk revealed for {}", self.source)
    }
}
