use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::bond::TxBondParametersDto, utils::value::Value,
};

use crate::{
    entity::Alias,
    hooks::{check_bond::CheckBond, check_step::CheckStep, query_validators::QueryValidatorSet},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Bond {
    pub source: Alias,
    pub amount: u64,
}

impl Step for Bond {
    fn to_json(&self, step_index: u64) -> StepType {
        StepType::Bond {
            parameters: TxBondParametersDto {
                source: Value::v(self.source.to_string()),
                validator: Value::r(step_index - 1, "validator-1-address".to_string()),
                amount: Value::v(self.amount.to_string()),
            },
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.source, &Alias::native_token(), self.amount);
        state.insert_bond(&self.source, self.amount);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(CheckBond::new(
                self.source.clone(),
                step_index - 1,
                self.amount,
            )),
        ]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryValidatorSet::new())]
    }
}

impl Display for Bond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bond {} from {}", self.amount, self.source)
    }
}
