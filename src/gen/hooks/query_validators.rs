use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType,
};

use crate::step::Hook;

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct QueryValidatorSet {}

impl Default for QueryValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryValidatorSet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hook for QueryValidatorSet {
    fn to_json(&self) -> StepType {
        todo!()
    }
}

impl Display for QueryValidatorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query validator set")
    }
}
