use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::change_consensus_key::TxChangeConsensusKeyParametersDto,
    utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::check_step::CheckStep,
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct ChangeConsensusKey {
    pub source: Alias,
    pub tx_settings: TxSettings,
}

impl Step for ChangeConsensusKey {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::ChangeConsensusKey {
            parameters: TxChangeConsensusKeyParametersDto {
                source: Value::v(self.source.to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_fees(&self.tx_settings)
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }

    fn total_post_hooks(&self) -> u64 {
        1
    }

    fn total_pre_hooks(&self) -> u64 {
        0
    }
}

impl Display for ChangeConsensusKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "change consensus key for {}", self.source)
    }
}
