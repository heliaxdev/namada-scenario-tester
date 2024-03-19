use std::fmt::Display;

use crate::{entity::Alias, step::Step};
use derive_builder::Builder;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Builder)]
pub struct NewWalletStep {
    pub alias: Alias,
}

impl Step for NewWalletStep {
    fn to_json(&self) -> String {
        todo!()
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_new_key(self.alias.clone())
    }

    fn post_hooks(&self, _step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }

    fn pre_hooks(&self, _step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for NewWalletStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wallet key with alias {}", self.alias)
    }
}
