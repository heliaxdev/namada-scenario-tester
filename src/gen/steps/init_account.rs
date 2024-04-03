use std::{collections::BTreeSet, fmt::Display};

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::init_account::TxInitAccountParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct InitAccount {
    pub alias: Alias,
    pub pks: BTreeSet<Alias>,
    pub threshold: u64,
    pub tx_settings: TxSettings,
}

impl Step for InitAccount {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::InitAccount {
            parameters: TxInitAccountParametersDto {
                alias: Value::v(self.alias.to_string()),
                sources: self
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
        state.add_new_account(self.alias.clone(), self.pks.clone(), self.threshold);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
    }

    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let gas_payer_balance =
            state.get_alias_token_balance(&self.tx_settings.gas_payer, &Alias::native_token());

        vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(CheckBalance::new(
                self.tx_settings.gas_payer.clone(),
                Alias::native_token(),
                gas_payer_balance,
            )),
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
