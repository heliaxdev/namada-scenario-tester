use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value, sdk::namada::Sdk,
};

use super::{Query, QueryParam};

#[derive(Clone, Debug, Default)]
pub struct AccountQuery {}

impl AccountQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for AccountQuery {
    type P = AccountQueryParameters;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, _state: &Storage) -> StepResult {
        let wallet = sdk.namada.wallet.read().await;

        let owner_address = wallet.find_address(&paramaters.address.alias);
        let owner_address = if let Some(address) = owner_address {
            address
        } else {
            return StepResult::fail() 
        };

        let account_info = rpc::get_account_info(
            sdk.namada.client(),
            owner_address,
        )
        .await;

        let account_info = if let Ok(Some(account)) = account_info {
            account
        } else {
            return StepResult::fail() 
        };

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), paramaters.address.alias);
        storage.add("address-alias".to_string(), owner_address.to_string());
        storage.add("pk-to-idx".to_string(), serde_json::to_string(&account_info.public_keys_map.pk_to_idx).unwrap());
        storage.add("threshold".to_string(), account_info.threshold.to_string());

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
