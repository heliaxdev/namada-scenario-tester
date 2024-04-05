use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::bond::TxBondParametersDto, utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{
        check_balance::CheckBalance, check_bond::CheckBond, check_step::CheckStep,
        query_validators::QueryValidatorSet,
    },
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct Bond {
    pub source: Alias,
    pub amount: u64,
    pub tx_settings: TxSettings,
}

impl Step for Bond {
    fn to_step_type(&self, step_index: u64) -> StepType {
        StepType::Bond {
            parameters: TxBondParametersDto {
                source: Value::v(self.source.to_string()),
                validator: Value::f(Some(step_index - 1)),
                amount: Value::v(self.amount.to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_token_balance(&self.source, &Alias::native_token(), self.amount);
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
        state.insert_bond(&self.source, self.amount);
    }

    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let bond_amount = state.get_account_total_bonded(&self.source);
        let source_balance = state.get_alias_token_balance(&self.source, &Alias::native_token());
        let mut hooks: Vec<Box<dyn crate::step::Hook>> = vec![
            Box::new(CheckStep::new(step_index)),
            // Box::new(CheckBond::new(self.source.clone(), step_index, bond_amount)),
            // Box::new(CheckBalance::new(
            //     self.source.clone(),
            //     Alias::native_token(),
            //     source_balance,
            // )),
        ];

        // if self.source.ne(&self.tx_settings.gas_payer) {
        //     let gas_payer_balance =
        //         state.get_alias_token_balance(&self.tx_settings.gas_payer, &Alias::native_token());
        //     hooks.push(Box::new(CheckBalance::new(
        //         self.tx_settings.gas_payer.clone(),
        //         Alias::native_token(),
        //         gas_payer_balance,
        //     )));
        // }
        hooks
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![Box::new(QueryValidatorSet::new())]
    }

    fn total_post_hooks(&self) -> u64 {
        if self.source.eq(&self.tx_settings.gas_payer) {
            1
        } else {
            1
        }
    }

    fn total_pre_hooks(&self) -> u64 {
        1
    }
}

impl Display for Bond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bond {} from {}", self.amount, self.source)
    }
}
