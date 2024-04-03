use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::init_pgf_steward_proposal::TxInitPgfStewardProposalParametersDto,
    utils::value::Value,
};

use crate::{
    constants::PROPOSAL_FUNDS,
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct InitPgfStewardProposal {
    pub author: Alias,
    pub start_epoch: Option<u64>,
    pub end_epoch: Option<u64>,
    pub grace_epoch: Option<u64>,
    pub steward_remove: Vec<Alias>,
    pub tx_settings: TxSettings,
}

impl Step for InitPgfStewardProposal {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::InitStewardProposal {
            parameters: TxInitPgfStewardProposalParametersDto {
                signer: Value::v(self.author.to_string()),
                start_epoch: self.start_epoch.map(|v| Value::v(v.to_string())),
                end_epoch: self.end_epoch.map(|v| Value::v(v.to_string())),
                grace_epoch: self.grace_epoch.map(|v| Value::v(v.to_string())),
                steward_remove: self
                    .steward_remove
                    .iter()
                    .map(|alias| Value::v(alias.to_string()))
                    .collect(),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.author, &Alias::native_token(), PROPOSAL_FUNDS);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
        state.last_proposal_id += 1;
    }

    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let author_balance = state.get_alias_token_balance(&self.author, &Alias::native_token());

        let mut hooks: Vec<Box<dyn crate::step::Hook>> = vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(CheckBalance::new(
                self.author.clone(),
                Alias::native_token(),
                author_balance,
            )),
        ];

        if self.author.ne(&self.tx_settings.gas_payer) {
            let gas_payer_balance =
                state.get_alias_token_balance(&self.author, &Alias::native_token());
            hooks.push(Box::new(CheckBalance::new(
                self.tx_settings.gas_payer.clone(),
                Alias::native_token(),
                gas_payer_balance,
            )));
        };
        hooks
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }

    fn total_post_hooks(&self) -> u64 {
        if self.author.eq(&self.tx_settings.gas_payer) {
            2
        } else {
            3
        }
    }

    fn total_pre_hooks(&self) -> u64 {
        0
    }
}

impl Display for InitPgfStewardProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "init pgf steward proposal by author {}", self.author)
    }
}
