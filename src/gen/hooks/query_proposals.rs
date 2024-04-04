use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{queries::proposals::ProposalsQueryParametersDto, scenario::StepType};

use crate::step::Hook;

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct QueryProposals {}

impl Default for QueryProposals {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryProposals {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hook for QueryProposals {
    fn to_step_type(&self) -> StepType {
        StepType::QueryProposals {
            parameters: ProposalsQueryParametersDto {},
        }
    }
}

impl Display for QueryProposals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query past proposals")
    }
}
