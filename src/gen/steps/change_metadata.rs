use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::change_metadata::TxChangeMetadataParametersDto, utils::value::Value,
};

use crate::{entity::Alias, hooks::check_step::CheckStep, state::State, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct ChangeMetadata {
    pub source: Alias,
}

impl Step for ChangeMetadata {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::ChangeMetadata {
            parameters: TxChangeMetadataParametersDto {
                source: Value::v(self.source.to_string()),
                email: Some(Value::f(None)),
                avatar: Some(Value::f(None)),
                commission_rate: Some(Value::f(None)),
                description: Some(Value::f(None)),
                discord_handle: Some(Value::f(None)),
                website: Some(Value::f(None)),
            },
            settings: None,
        }
    }

    fn update_state(&self, _state: &mut crate::state::State) {}

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for ChangeMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "metadata change for {}", self.source)
    }
}
