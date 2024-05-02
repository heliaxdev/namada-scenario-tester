use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

pub enum BalanceQueryStorageKeys {
    Address,
    Amount,
    TokenAddress,
}

impl ToString for BalanceQueryStorageKeys {
    fn to_string(&self) -> String {
        match self {
            BalanceQueryStorageKeys::Address => "address".to_string(),
            BalanceQueryStorageKeys::Amount => "amount".to_string(),
            BalanceQueryStorageKeys::TokenAddress => "token-address".to_string(),
        }
    }
}

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

        let balance = match balance {
            Ok(balance) => balance.to_string(),
            Err(e) => return StepResult::fail(e.to_string()),
        };

        let mut storage = StepStorage::default();
        storage.add(
            BalanceQueryStorageKeys::Address.to_string(),
            owner_address.to_string(),
        );
        storage.add(
            BalanceQueryStorageKeys::Amount.to_string(),
            balance.to_string(),
        );
        storage.add(
            BalanceQueryStorageKeys::TokenAddress.to_string(),
            token_address.to_string(),
        );

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceQueryParametersDto {
    pub address: Value,
    pub token: Value,
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
            Value::Ref { value, field } => {
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    "public-key" => AccountIndentifier::PublicKey(data),
                    "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                    _ => AccountIndentifier::Address(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value, field } => {
                let address = state.get_step_item(&value, &field);
                AccountIndentifier::Address(address)
            }
            Value::Value { value } => AccountIndentifier::Alias(value),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Self { address, token }
    }
}
