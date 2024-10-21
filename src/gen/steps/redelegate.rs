use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::redelegate::TxRedelegateParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_step::CheckStep, query_validators::QueryValidatorSet},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Redelegate {
    pub source: Alias,
    pub source_validator: u64, // step id of a bond step
    pub amount: u64,
    pub tx_settings: TxSettings,
}

impl Step for Redelegate {
    fn to_step_type(&self, step_index: u64) -> StepType {
        StepType::Redelegate {
            parameters: TxRedelegateParametersDto {
                source: Value::v(self.source.to_string()),
                src_validator: Value::r(self.source_validator, "validator-0-address".to_string()),
                dest_validator: Value::f(Some(step_index - 1)),
                amount: Value::v(self.amount.to_string()), // amount: Value::r(self.source_validator, "amount-0".to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.insert_redelegation_and_update_bonds(
            &self.source,
            self.source_validator,
            self.amount,
        );
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

impl Display for Redelegate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "redelegate {} for {}", self.amount, self.source)
    }
}
