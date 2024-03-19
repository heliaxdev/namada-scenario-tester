use std::fmt::Display;

use derive_builder::Builder;

use crate::{entity::Alias, hooks::check_step::CheckStep, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct TransparentTransfer {
    pub source: Alias,
    pub target: Alias,
    pub token: Alias,
    pub amount: u64,
}

impl Step for TransparentTransfer {
    fn to_json(&self) -> String {
        todo!()
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.source, &self.token, self.amount);
        state.increase_account_token_balance(&self.target, self.token.clone(), self.amount);
    }

    fn post_hooks(&self, step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for TransparentTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "transfer {} {} from {} to {}",
            self.amount, self.token, self.source, self.target
        )
    }
}
