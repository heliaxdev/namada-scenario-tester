use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::redelegate::TxRedelegateParametersDto, utils::value::Value,
};

use crate::{
    entity::Alias,
    hooks::{check_bond::CheckBond, check_step::CheckStep, query_validators::QueryValidatorSet},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Redelegate {
    pub source: Alias,
    pub source_validator: u64, // step id of a bond step
    pub amount: u64,
}

impl Step for Redelegate {
    fn to_step_type(&self, step_index: u64) -> StepType {
        StepType::Redelegate {
            parameters: TxRedelegateParametersDto {
                source: Value::v(self.source.to_string()),
                src_validator: Value::r(self.source_validator, "validator-address".to_string()),
                dest_validator: Value::f(Some(step_index - 1)),
                amount: Value::v(self.amount.to_string()),
            },
            settings: None,
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.update_bonds_by_redelegation(&self.source, self.source_validator, self.amount);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(CheckBond::new(self.source.clone(), step_index, self.amount)),
        ]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryValidatorSet::new())]
    }
}

impl Display for Redelegate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "redelegate {} for {}", self.amount, self.source)
    }
}
