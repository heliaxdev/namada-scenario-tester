use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::bond_batch::TxBondBatchParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_step::CheckStep, query_validators::QueryValidatorSet},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct BondBatch {
    pub sources: Vec<Alias>,
    pub amounts: Vec<u64>,
    pub tx_settings: TxSettings,
}

impl Step for BondBatch {
    fn to_step_type(&self, step_index: u64) -> StepType {
        StepType::BondBatch {
            parameters: TxBondBatchParametersDto {
                sources: self
                    .sources
                    .iter()
                    .map(|source| Value::v(source.to_string()))
                    .collect(),
                targets: self
                    .sources
                    .iter()
                    .map(|_| Value::f(Some(step_index - 1)))
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
            let source = self.sources[idx].clone();
            let amount = self.amounts[idx];
            state.decrease_account_token_balance(&source, &Alias::native_token(), amount);
            state.insert_bond(&source, amount);
        }
        // let source = self.sources[0].clone();
        // let amount = self.amounts[0];
        // state.decrease_account_token_balance(&source, &Alias::native_token(), amount);
        // state.insert_bond(&source, amount);
        state.decrease_account_fees(&self.tx_settings);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryValidatorSet::new())]
    }

    fn total_post_hooks(&self) -> u64 {
        1
    }

    fn total_pre_hooks(&self) -> u64 {
        1
    }
}

impl Display for BondBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bond batch of size {}", self.sources.len())
    }
}
