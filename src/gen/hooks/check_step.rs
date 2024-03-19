use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    checks::step::StepCheckParametersDto, scenario::StepType, utils::value::Value,
};

use crate::step::Hook;

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckStep {
    inner: u64,
}

impl CheckStep {
    pub fn new(step: u64) -> Self {
        Self { inner: step }
    }
}

impl Hook for CheckStep {
    fn to_json(&self) -> StepType {
        StepType::CheckStepOutput {
            parameters: StepCheckParametersDto {
                outcome: Value::v("success".to_string()),
                id: Value::v(self.inner.to_string()),
            },
        }
    }
}

impl Display for CheckStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check step {}", self.inner)
    }
}
