use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::tx_transparent_transfer::TxTransparentTransferParametersDto,
    utils::value::Value,
};

use crate::{entity::Alias, hooks::check_step::CheckStep, state::State, step::Step};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct FaucetTransfer {
    pub target: Alias,
    pub token: Alias,
    pub amount: u64,
}

impl Step for FaucetTransfer {
    fn to_step_type(&self, _step_index: u64) -> StepType {
        StepType::TransparentTransfer {
            parameters: TxTransparentTransferParametersDto {
                source: Value::v("faucet".to_string()),
                target: Value::v(self.target.to_string()),
                amount: Value::v(self.amount.to_string()),
                token: Value::v(self.token.to_string()),
            },
            settings: None,
        }
    }

    fn update_state(&self, state: &mut crate::state::State) {
        state.increase_account_token_balance(&self.target, self.token.clone(), self.amount);
    }

    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn crate::step::Hook>> {
        let _target_balance = state.get_alias_token_balance(&self.target, &self.token);
        vec![
            Box::new(CheckStep::new(step_index)),
            // Box::new(CheckBalance::new(
            //     self.target.clone(),
            //     self.token.clone(),
            //     target_balance,
            // )),
        ]
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

impl Display for FaucetTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "transfer {} {} from faucet to {}",
            self.amount, self.token, self.target
        )
    }
}
