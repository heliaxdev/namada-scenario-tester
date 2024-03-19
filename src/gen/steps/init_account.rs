use std::fmt::Display;

use derive_builder::Builder;

use crate::{entity::Alias, hooks::check_step::CheckStep, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct InitAccount {
    pub alias: Alias,
    pub pks: Vec<Alias>,
    pub threshold: u64,
}

impl Step for InitAccount {
    fn to_json(&self) -> String {
        todo!()
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.add_new_account(self.alias.clone(), self.pks.clone(), self.threshold);
    }

    fn post_hooks(&self, step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(CheckStep::new(step_index))]
    }

    fn pre_hooks(&self, _step_index: u64) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }
}

impl Display for InitAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "init account {} with {} and treshold {}",
            self.alias,
            self.pks.len(),
            self.threshold
        )
    }
}
