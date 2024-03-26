use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::become_validator::BecomeValidatorParametersDto, utils::value::Value,
};

use crate::{entity::Alias, hooks::check_step::CheckStep, state::State, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct BecomeValidator {
    pub source: Alias,
}

impl Step for BecomeValidator {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::BecomeValidator {
            parameters: BecomeValidatorParametersDto {
                source: Value::v(self.source.to_string()),
                commission_rate: Value::f(None),
            },
            settings: None,
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.set_account_as_validator(&self.source);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for BecomeValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "become validator for {}", self.source)
    }
}
