use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::vote::TxVoteProposalParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_step::CheckStep, query_proposals::QueryProposals},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct VoteProposal {
    pub voter: Alias,
    pub tx_settings: TxSettings,
}

impl Step for VoteProposal {
    fn to_step_type(&self, step_index: u64) -> StepType {
        StepType::VoteProposal {
            parameters: TxVoteProposalParametersDto {
                proposal_id: Value::f(Some(step_index - 1)),
                voter: Value::v(self.voter.to_string()),
                vote: Value::f(None),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        // TODO: we can't tell if the transaction will be executed or not, but we subsract the fees anyway
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryProposals::new())]
    }

    fn total_post_hooks(&self) -> u64 {
        1
    }

    fn total_pre_hooks(&self) -> u64 {
        1
    }
}

impl Display for VoteProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vote proposal from {}", self.voter)
    }
}
