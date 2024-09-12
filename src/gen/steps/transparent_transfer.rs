use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::tx_transparent_transfer::TxTransparentTransferParametersDto,
    utils::value::Value,
};

use crate::{
    entity::{Alias, TxSettings},
    hooks::{check_balance::CheckBalance, check_step::CheckStep, query_balance::QueryBalance},
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
        state.decrease_account_fees(&self.tx_settings);
        state.decrease_account_token_balance(&self.source, &self.token, self.amount);
        state.increase_account_token_balance(&self.target, self.token.clone(), self.amount);
    }

    fn post_hooks(&self, step_index: u64, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let check_balance_source = CheckBalance::new(
            step_index - 2,
            self.source.clone(),
            self.token.clone(),
            "le".to_string(),
        );
        let check_balance_target = CheckBalance::new(
            step_index - 1,
            self.target.clone(),
            self.token.clone(),
            "ge".to_string(),
        );
        vec![
            Box::new(CheckStep::new(step_index)),
            Box::new(check_balance_source),
            Box::new(check_balance_target),
        ]
    }

    fn pre_hooks(&self, _state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let query_balance_source = QueryBalance::new(self.source.to_owned(), Alias::native_token());
        let query_balance_target = QueryBalance::new(self.target.to_owned(), Alias::native_token());
        vec![
            Box::new(query_balance_source),
            Box::new(query_balance_target),
        ]
    }

    fn total_post_hooks(&self) -> u64 {
        3
    }

    fn total_pre_hooks(&self) -> u64 {
        2
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
