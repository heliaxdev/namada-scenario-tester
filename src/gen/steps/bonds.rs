use std::fmt::Display;

use derive_builder::Builder;

use crate::{
    entity::Alias,
    hooks::{check_step::CheckStep, query_validators::QueryValidatorSet},
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Bond {
    pub source: Alias,
    pub amount: u64,
}

impl Step for Bond {
    fn to_json(&self) -> String {
        todo!()
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.source, &Alias::native_token(), self.amount);
        state.insert_bond(&self.source, self.amount);
    }

    fn post_hooks(&self, step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryValidatorSet::new())]
    }
}

impl Display for Bond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bond {} from {}", self.amount, self.source)
    }
}
