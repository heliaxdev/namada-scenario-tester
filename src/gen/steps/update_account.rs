use std::{collections::BTreeSet, fmt::Display};

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::update_account::TxUpdateAccountParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::check_step::CheckStep,
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct UpdateAccount {
    pub source: Alias,
    pub pks: BTreeSet<Alias>,
    pub threshold: u64,
    pub tx_settings: TxSettings,
}

impl Step for UpdateAccount {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::UpdateAccount {
            parameters: TxUpdateAccountParametersDto {
                source: Value::v(self.source.to_string()),
                public_keys: self
                    .pks
                    .iter()
                    .map(|alias| Value::v(alias.to_string()))
                    .collect(),
                threshold: Some(Value::v(self.threshold.to_string())),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.modify_new_account(self.source.clone(), self.pks.clone(), self.threshold);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
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

impl Display for UpdateAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "update account {} with {} and treshold {}",
            self.source,
            self.pks.len(),
            self.threshold
        )
    }
}
