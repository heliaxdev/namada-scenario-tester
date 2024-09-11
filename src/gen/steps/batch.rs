use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{scenario::StepType, tasks::batch::BatchParameterDto};
use rand::{distributions::Standard, prelude::Distribution};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_step::CheckStep, query_validators::QueryValidatorSet},
    state::State,
    step::Step,
};

use super::{bonds::Bond, transparent_transfer::TransparentTransfer};

#[derive(Clone, Debug)]
pub enum BatchInnerType {
    TransparentTransfer,
    Bond,
}

impl Distribution<BatchInnerType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BatchInnerType {
        match rng.gen_range(0..=1) {
            0 => BatchInnerType::TransparentTransfer,
            _ => BatchInnerType::Bond,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchInner {
    TransparentTransfer(TransparentTransfer),
    Bond(Bond),
}

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Batch {
    pub batch: Vec<BatchInner>,
    pub tx_settings: TxSettings,
}

impl Step for Batch {
    fn to_step_type(&self, step_index: u64) -> StepType {
        let parameters = self
            .batch
            .iter()
            .map(|step_type| match step_type {
                BatchInner::TransparentTransfer(p) => {
                    let parameters = match p.to_step_type(step_index) {
                        StepType::TransparentTransfer { parameters, .. } => parameters,
                        _ => unreachable!(),
                    };
                    BatchParameterDto::TransparentTransfer(parameters)
                }
                BatchInner::Bond(p) => {
                    let parameters = match p.to_step_type(step_index) {
                        StepType::Bond { parameters, .. } => parameters,
                        _ => unreachable!(),
                    };
                    BatchParameterDto::Bond(parameters)
                }
            })
            .collect();

        StepType::Batch {
            parameters,
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut State) {
        for tx in self.batch.iter() {
            match tx {
                BatchInner::TransparentTransfer(tx) => {
                    state.decrease_account_token_balance(&tx.source, &tx.token, tx.amount);
                    state.increase_account_token_balance(&tx.target, tx.token.clone(), tx.amount);
                }
                BatchInner::Bond(tx) => {
                    state.decrease_account_token_balance(
                        &tx.source,
                        &Alias::native_token(),
                        tx.amount,
                    );
                    state.insert_bond(&tx.source, tx.amount);
                }
            }
        }
        state.decrease_account_fees(&self.tx_settings)
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

impl Display for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "batch ({})", self.batch.len())
    }
}
