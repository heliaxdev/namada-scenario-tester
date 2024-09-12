use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::transparent_transfer_batch::TxTransparentTransferBatchParametersDto,
    utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::check_step::CheckStep,
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct TransparentTransferBatch {
    pub sources: Vec<Alias>,
    pub targets: Vec<Alias>,
    pub tokens: Vec<Alias>,
    pub amounts: Vec<u64>,
    pub tx_settings: TxSettings,
}

impl Step for TransparentTransferBatch {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::TransparentTransferBatch {
            parameters: TxTransparentTransferBatchParametersDto {
                sources: self
                    .sources
                    .iter()
                    .map(|source| Value::v(source.to_string()))
                    .collect(),
                targets: self
                    .targets
                    .iter()
                    .map(|target| Value::v(target.to_string()))
                    .collect(),
                tokens: self
                    .tokens
                    .iter()
                    .map(|token| Value::v(token.to_string()))
                    .collect(),
                amounts: self
                    .amounts
                    .iter()
                    .map(|amount| Value::v(amount.to_string()))
                    .collect(),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        for idx in 0..self.sources.len() {
            let token = self.tokens[idx].clone();
            let source = self.sources[idx].clone();
            let target = self.targets[idx].clone();
            let amount = self.amounts[idx];
            state.decrease_account_token_balance(&source, &token, amount);
            state.increase_account_token_balance(&target, token, amount);
        }
        state.decrease_account_fees(&self.tx_settings);
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

impl Display for TransparentTransferBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "transparent batch transfer of size {}",
            self.sources.len()
        )
    }
}
