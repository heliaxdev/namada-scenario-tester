use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::become_validator::BecomeValidatorParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct BecomeValidator {
    pub source: Alias,
    pub tx_settings: TxSettings,
}

impl Step for BecomeValidator {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::BecomeValidator {
            parameters: BecomeValidatorParametersDto {
                source: Value::v(self.source.to_string()),
                commission_rate: Value::f(None),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.set_account_as_validator(&self.source);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None)
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

impl Display for BecomeValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "become validator for {}", self.source)
    }
}
