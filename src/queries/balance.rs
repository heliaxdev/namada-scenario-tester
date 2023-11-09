use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

#[derive(Clone, Debug, Default)]
pub struct BalanceQuery {}

impl BalanceQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for BalanceQuery {
    type P = BalanceQueryParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let owner_address = parameters.address.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let balance =
            rpc::get_token_balance(sdk.namada.client(), &token_address, &owner_address).await;

        let balance = if let Ok(balance) = balance {
            balance.to_string_native()
        } else {
            return StepResult::fail();
        };

        let mut storage = StepStorage::default();
        storage.add("address".to_string(), owner_address.to_string());
        storage.add("amount".to_string(), balance.to_string());

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceQueryParametersDto {
    address: Value,
    token: Value,
}

#[derive(Clone, Debug)]
pub struct BalanceQueryParameters {
    pub address: AccountIndentifier,
    pub token: AccountIndentifier,
}

impl QueryParam for BalanceQueryParameters {
    type D = BalanceQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
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

        Self { address, token }
    }
}
