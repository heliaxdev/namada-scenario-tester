use async_trait::async_trait;

use namada_sdk::{rpc, Namada};
use serde::Deserialize;

use crate::entity::address::{AccountIndentifier, ADDRESS_PREFIX};
use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct BalanceCheck {}

impl BalanceCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for BalanceCheck {
    type P = BalanceCheckParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let owner_address = parameters.address.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let balance =
            rpc::get_token_balance(sdk.namada.client(), &token_address, &owner_address).await;

        let balance = balance.unwrap().to_string_native();

        if parameters.amount.to_string().eq(&balance) {
            StepResult::success_empty()
        } else {
            StepResult::fail()
        }
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
    address: AccountIndentifier,
    token: AccountIndentifier,
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
                AccountIndentifier::StateAddress(state.get_address(&alias))
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz {} => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value } => {
                let address = state.get_step_item(&value, "token-address");
                AccountIndentifier::Address(address)
            }
            Value::Value { value } => AccountIndentifier::Alias(value),
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            amount,
            address,
            token,
        }
    }
}
