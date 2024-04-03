use std::fmt::Display;

use crate::{
    entity::Alias,
    hooks::{check_reveal_pk::CheckRevealPk, reveal_pk::RevealPk},
    state::State,
    step::Step,
};
use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::wallet_new_key::WalletNewKeyParametersDto, utils::value::Value,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Builder)]
pub struct NewWalletStep {
    pub alias: Alias,
}

impl Step for NewWalletStep {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::WalletNewKey {
            parameters: WalletNewKeyParametersDto {
                alias: Value::v(self.alias.to_string()),
            },
            settings: None,
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_new_key(self.alias.clone())
    }

    fn post_hooks(&self, _step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![
            Box::new(RevealPk::new(self.alias.clone())),
            Box::new(CheckRevealPk::new(self.alias.clone())),
        ]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }

    fn total_post_hooks(&self) -> u64 {
        2
    }

    fn total_pre_hooks(&self) -> u64 {
        0
    }
}

impl Display for NewWalletStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wallet key with alias {}", self.alias)
    }
}
