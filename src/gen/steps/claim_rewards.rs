use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::claim_rewards::TxClaimRewardsteParametersDto, utils::value::Value,
};

use crate::{entity::TxSettings, hooks::check_step::CheckStep, state::State, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct ClaimRewards {
    pub bond_step: u64, // step id of a bond step
    pub tx_settings: TxSettings,
}

impl Step for ClaimRewards {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::ClaimRewards {
            parameters: TxClaimRewardsteParametersDto {
                source: Value::r(self.bond_step, "validator-address".to_string()),
                delegator: Value::r(self.bond_step, "source-address".to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
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

impl Display for ClaimRewards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "claim rewards")
    }
}
