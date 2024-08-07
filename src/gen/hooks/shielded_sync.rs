use std::fmt::Display;

use namada_scenario_tester::scenario::StepType;

use crate::{entity::Alias, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShieldedSync {}

impl ShieldedSync {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hook for ShieldedSync {
    fn to_step_type(&self) -> StepType {
        StepType::ShieldedSync
    }
}

impl Display for ShieldedSync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "synchronizing the state of the masp")
    }
}