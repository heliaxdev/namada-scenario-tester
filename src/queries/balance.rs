use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value, sdk::namada::Sdk,
};

use super::{Query, QueryParam};

#[derive(Clone, Debug, Default)]
pub struct BalanceQuery {
    rpc: String,
    chain_id: String,
}

impl BalanceQuery {
    pub fn new(sdk: &Sdk) -> Self {
        Self {
            rpc: sdk.rpc.clone(),
            chain_id: sdk.chain_id.clone(),
        }
    }
}

impl Query for BalanceQuery {
    type P = BalanceQueryParameters;

    fn execute(&self, paramaters: Self::P, _state: &Storage) -> StepResult {
        println!(
            "namada client balance --owner {} --token {}",
            paramaters.address.alias, paramaters.token
        );

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), paramaters.address.alias);
        storage.add("amount".to_string(), "500".to_string());
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

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
