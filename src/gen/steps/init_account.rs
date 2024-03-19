use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::init_account::TxInitAccountParametersDto, utils::value::Value,
};

use crate::{entity::Alias, hooks::check_step::CheckStep, state::State, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct InitAccount {
    pub alias: Alias,
    pub pks: Vec<Alias>,
    pub threshold: u64,
}

impl Step for InitAccount {
    fn to_json(&self, _step_index: u64) -> StepType {
        StepType::InitAccount {
            parameters: TxInitAccountParametersDto {
                sources: self
                    .pks
                    .iter()
                    .map(|alias| Value::v(alias.to_string()))
                    .collect(),
                threshold: Some(Value::v(self.threshold.to_string())),
            },
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.add_new_account(self.alias.clone(), self.pks.clone(), self.threshold);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for InitAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "init account {} with {} and treshold {}",
            self.alias,
            self.pks.len(),
            self.threshold
        )
    }
}
