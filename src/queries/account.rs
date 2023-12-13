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

pub enum AccountQueryStorageKeys {
    Address,
    Threshold,
    TotalPublicKeys,
    PublicKeyAtIndex(u8),
}

impl ToString for AccountQueryStorageKeys {
    fn to_string(&self) -> String {
        match self {
            AccountQueryStorageKeys::Address => "address".to_string(),
            AccountQueryStorageKeys::Threshold => "threshold".to_string(),
            AccountQueryStorageKeys::TotalPublicKeys => "total_public_keys".to_string(),
            AccountQueryStorageKeys::PublicKeyAtIndex(index) => {
                format!("public_key_at_index-{}", index)
            }
        }
    }
}

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

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let owner_address = parameters.address.to_namada_address(sdk).await;

        let account_info = rpc::get_account_info(sdk.namada.client(), &owner_address).await;

        let account_info = if let Ok(Some(account)) = account_info {
            account
        } else {
            return StepResult::fail();
        };

        let mut storage = StepStorage::default();
        storage.add(
            AccountQueryStorageKeys::Address.to_string(),
            owner_address.to_string(),
        );
        storage.add(
            AccountQueryStorageKeys::Threshold.to_string(),
            account_info.threshold.to_string(),
        );
        storage.add(
            AccountQueryStorageKeys::TotalPublicKeys.to_string(),
            account_info.public_keys_map.idx_to_pk.len().to_string(),
        );
        for (key, value) in account_info.public_keys_map.idx_to_pk.into_iter() {
            storage.add(
                AccountQueryStorageKeys::PublicKeyAtIndex(key).to_string(),
                value.to_string(),
            );
        }

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountQueryParametersDto {
    address: Value,
}

#[derive(Clone, Debug)]
pub struct AccountQueryParameters {
    address: AccountIndentifier,
}

impl QueryParam for AccountQueryParameters {
    type D = AccountQueryParametersDto;

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
            Value::Fuzz {} => unimplemented!(),
        };

        Self { address }
    }
}
