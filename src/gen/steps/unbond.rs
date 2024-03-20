use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::unbond::TxUnbondParametersDto, utils::value::Value,
};

use crate::{
    constants::BOND_VALIDATOR_STORAGE_KEY, entity::Alias, hooks::check_step::CheckStep,
    state::State, step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Unbond {
    pub source: Alias,
    pub amount: u64,
    pub bond_step: u64,
}

impl Step for Unbond {
    fn to_json(&self, _step_index: u64) -> StepType {
        StepType::Unbond {
            parameters: TxUnbondParametersDto {
                source: Value::v(self.source.to_string()),
                validator: Value::r(self.bond_step, BOND_VALIDATOR_STORAGE_KEY.to_string()),
                amount: Value::v(self.amount.to_string()),
            },
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_unbond(&self.source, self.amount, self.bond_step);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for Unbond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unbond {} for {}", self.amount, self.source)
    }
}
