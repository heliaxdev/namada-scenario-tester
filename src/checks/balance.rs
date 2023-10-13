use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, Storage},
    utils::value::Value,
};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct BalanceCheck {}

impl Check for BalanceCheck {
    type P = BalanceCheckParameters;

    fn execute(&self, paramaters: Self::P, _state: &Storage) -> StepResult {
        println!(
            "namada client balance --owner {} --token {}",
            paramaters.address.alias, paramaters.token
        );
        // TODO: check balance
        StepResult::success_empty()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceCheckParametersDto {
    amount: Value,
    address: Value,
    token: Value,
}

#[derive(Clone, Debug)]
pub struct BalanceCheckParameters {
    amount: u64,
    address: Address,
    token: String,
}

impl CheckParam for BalanceCheckParameters {
    type D = BalanceCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let amount = match dto.amount {
            Value::Ref { value } => state
                .get_step_item(&value, "amount")
                .parse::<u64>()
                .unwrap(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };
        let address = match dto.address {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value } => state.get_step_item(&value, "token"),
            Value::Value { value } => value.to_owned(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            amount,
            address,
            token,
        }
    }
}
