use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::scenario::StepType;

use crate::step::Hook;

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckStorage {
    field: String,
    value: String,
    step: u64,
}

impl CheckStorage {
    pub fn new(field: String, value: String, step: u64) -> Self {
        Self { field, value, step }
    }
}

impl Hook for CheckStorage {
    fn to_step_type(&self) -> StepType {
        todo!()
    }
}

impl Display for CheckStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "check storage field {} for step {}",
            self.field, self.step
        )
    }
}
