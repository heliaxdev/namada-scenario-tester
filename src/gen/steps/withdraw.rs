use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::withdraw::TxWithdrawParametersDto, utils::value::Value,
};

use crate::{
    constants::UNBOND_VALIDATOR_STORAGE_KEY, entity::Alias, hooks::check_step::CheckStep,
    state::State, step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Withdraw {
    pub source: Alias,
    pub amount: u64,
    pub unbond_step: u64,
}

impl Step for Withdraw {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::Withdraw {
            parameters: TxWithdrawParametersDto {
                source: Value::v(self.source.to_string()),
                validator: Value::r(self.unbond_step, UNBOND_VALIDATOR_STORAGE_KEY.to_string()),
            },
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_withdraw(&self.source, self.amount, self.unbond_step);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for Withdraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "withdraw for {}", self.source)
    }
}
