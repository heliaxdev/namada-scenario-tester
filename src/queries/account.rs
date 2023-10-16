use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

#[derive(Clone, Debug, Default)]
pub struct AccountQuery {
    rpc: String,
    chain_id: String,
}

impl AccountQuery {
    pub fn new(rpc: String, chain_id: String) -> Self {
        Self { rpc, chain_id }
    }
}

impl Query for AccountQuery {
    type P = AccountQueryParameters;

    fn execute(&self, paramaters: Self::P, _state: &Storage) -> StepResult {
        println!(
            "namada client query-account --owner {}",
            paramaters.address.alias
        );

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), paramaters.address.alias);
        storage.add("keys".to_string(), "keys".to_string());
        storage.add("threshold".to_string(), "threshold".to_string());
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountQueryParametersDto {
    address: Value,
}

#[derive(Clone, Debug)]
pub struct AccountQueryParameters {
    address: Address,
}

impl QueryParam for AccountQueryParameters {
    type D = AccountQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let address = match dto.address {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };

        Self { address }
    }
}
