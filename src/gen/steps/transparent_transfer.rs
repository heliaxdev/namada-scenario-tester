use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::tx_transparent_transfer::TxTransparentTransferParametersDto,
    utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep},
    state::State,
    step::Step,
};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct TransparentTransfer {
    pub source: Alias,
    pub target: Alias,
    pub token: Alias,
    pub amount: u64,
    pub tx_settings: TxSettings,
}

impl Step for TransparentTransfer {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::TransparentTransfer {
            parameters: TxTransparentTransferParametersDto {
                source: Value::v(self.source.to_string()),
                target: Value::v(self.target.to_string()),
                amount: Value::v(self.amount.to_string()),
                token: Value::v(self.token.to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.decrease_account_fees(&self.tx_settings.gas_payer, &None);
        state.decrease_account_token_balance(&self.source, &self.token, self.amount);
        state.increase_account_token_balance(&self.target, self.token.clone(), self.amount);
    }

    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let target_balance = state.get_alias_token_balance(&self.target, &self.token);
        let source_balance = state.get_alias_token_balance(&self.source, &self.token);

        let mut hooks: Vec<Box<dyn crate::step::Hook>> = vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(CheckBalance::new(
                self.source.clone(),
                self.token.clone(),
                source_balance,
            )),
            Box::new(CheckBalance::new(
                self.target.clone(),
                self.token.clone(),
                target_balance,
            )),
        ];

        if self.source.ne(&self.tx_settings.gas_payer) {
            let gas_payer_balance =
                state.get_alias_token_balance(&self.tx_settings.gas_payer, &Alias::native_token());
            hooks.push(Box::new(CheckBalance::new(
                self.tx_settings.gas_payer.clone(),
                Alias::native_token(),
                gas_payer_balance,
            )));
        }
        hooks
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        vec![]
    }

    fn total_post_hooks(&self) -> u64 {
        if self.source.eq(&self.tx_settings.gas_payer) {
            3
        } else {
            4
        }
    }

    fn total_pre_hooks(&self) -> u64 {
        0
    }
}

impl Display for TransparentTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "transfer {} {} from {} to {}",
            self.amount, self.token, self.source, self.target
        )
    }
}
