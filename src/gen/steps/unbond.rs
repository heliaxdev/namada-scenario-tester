use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::unbond::TxUnbondParametersDto, utils::value::Value,
};

use crate::{
    constants::BOND_VALIDATOR_STORAGE_KEY,
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Unbond {
    pub source: Alias,
    pub amount: u64,
    pub bond_step: u64,
    pub tx_settings: TxSettings,
}

impl Step for Unbond {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::Unbond {
            parameters: TxUnbondParametersDto {
                source: Value::v(self.source.to_string()),
                validator: Value::r(self.bond_step, BOND_VALIDATOR_STORAGE_KEY.to_string()),
                amount: Value::v(self.amount.to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_unbond(&self.source, self.amount, self.bond_step);
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

impl Display for Unbond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unbond {} for {}", self.amount, self.source)
    }
}
