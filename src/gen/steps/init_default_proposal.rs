use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::init_default_proposal::TxInitDefaultProposalParametersDto,
    utils::value::Value,
};

use crate::{
    constants::PROPOSAL_FUNDS,
    entity::{Alias, TxSettings},
    hooks::check_step::CheckStep,
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct InitDefaultProposal {
    pub author: Alias,
    pub start_epoch: Option<u64>,
    pub end_epoch: Option<u64>,
    pub grace_epoch: Option<u64>,
    pub tx_settings: TxSettings,
}

impl Step for InitDefaultProposal {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::InitProposal {
            parameters: TxInitDefaultProposalParametersDto {
                signer: Value::v(self.author.to_string()),
                start_epoch: self.start_epoch.map(|v| Value::v(v.to_string())),
                end_epoch: self.end_epoch.map(|v| Value::v(v.to_string())),
                grace_epoch: self.grace_epoch.map(|v| Value::v(v.to_string())),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.author, &Alias::native_token(), PROPOSAL_FUNDS);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
        state.last_proposal_id += 1;
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

impl Display for InitDefaultProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "init default proposal by author {}", self.author)
    }
}
