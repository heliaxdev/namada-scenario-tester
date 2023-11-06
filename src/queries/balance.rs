use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::Deserialize;

use namada_sdk::core::types::address::Address as NamadaAddress;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value, sdk::namada::Sdk,
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

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, _state: &Storage) -> StepResult {
        let wallet = sdk.namada.wallet.read().await;

        let owner_address = wallet.find_address(&paramaters.address.alias);
        let owner_address = if let Some(address) = owner_address {
            address
        } else {
            return StepResult::fail() 
        };

        let balance = rpc::get_token_balance(
            sdk.namada.client(),
            &NamadaAddress::decode(&paramaters.token).unwrap(),
            &owner_address,
        )
        .await;

        let balance = if let Ok(balance) = balance {
            balance.to_string_native()
        } else {
            return StepResult::fail()
        };

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), paramaters.address.alias);
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
    address: Address,
    token: String,
}

impl QueryParam for BalanceQueryParameters {
    type D = BalanceQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
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

        Self { address, token }
    }
}
