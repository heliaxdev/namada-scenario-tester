use std::fmt::Display;

use async_trait::async_trait;

use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

pub enum ValidatorsQueryStorageKeys {
    Validator(u64),
    TotalValidator,
}

impl Display for ValidatorsQueryStorageKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidatorsQueryStorageKeys::Validator(index) => {
                write!(f, "validator-{}-address", index)
            }
            ValidatorsQueryStorageKeys::TotalValidator => write!(f, "total-validators"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValidatorsQuery {}

impl ValidatorsQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for ValidatorsQuery {
    type P = ValidatorsQueryParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let current_epoch = match parameters.epoch {
            Some(value) => namada_sdk::storage::Epoch::from(value),
            None => rpc::query_epoch(sdk.namada.client()).await.unwrap(),
        };
        let validators = rpc::get_all_validators(sdk.namada.client(), current_epoch)
            .await
            .unwrap();

        let mut storage = StepStorage::default();

        storage.add(
            ValidatorsQueryStorageKeys::TotalValidator.to_string(),
            validators.len().to_string(),
        );

        for (index, address) in validators.into_iter().enumerate() {
            storage.add(
                ValidatorsQueryStorageKeys::Validator(index as u64).to_string(),
                address.to_string(),
            );
        }

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorsQueryParametersDto {
    epoch: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct ValidatorsQueryParameters {
    pub epoch: Option<u64>,
}

impl QueryParam for ValidatorsQueryParameters {
    type D = ValidatorsQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let epoch = dto.epoch.map(|value| match value {
            Value::Ref { value, field } => {
                let data = state.get_step_item(&value, &field);
                data.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { value: _ } => unimplemented!(),
        });

        Self { epoch }
    }
}
