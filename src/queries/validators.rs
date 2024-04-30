use std::collections::BTreeSet;

use async_trait::async_trait;

use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{misc::ValidatorState, value::Value},
};

use super::{Query, QueryParam};

pub enum ValidatorsQueryStorageKeys {
    Validator(u64),
    State(u64),
    TotalValidator,
}

impl ToString for ValidatorsQueryStorageKeys {
    fn to_string(&self) -> String {
        match self {
            ValidatorsQueryStorageKeys::Validator(index) => format!("validator-{}-address", index),
            ValidatorsQueryStorageKeys::State(index) => format!("validator-{}-state", index),
            ValidatorsQueryStorageKeys::TotalValidator => "total-validators".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
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
        let validators: BTreeSet<_> = rpc::get_all_validators(sdk.namada.client(), current_epoch)
            .await
            .unwrap()
            .into_iter()
            .collect();

        let mut storage = StepStorage::default();

        storage.add(
            ValidatorsQueryStorageKeys::TotalValidator.to_string(),
            validators.len().to_string(),
        );

        for (index, address) in validators.into_iter().enumerate() {
            let validator_state = rpc::get_validator_state(sdk.namada.client(), &address, None)
                .await
                .unwrap();
            match validator_state {
                (Some(state), _) => {
                    storage.add(
                        ValidatorsQueryStorageKeys::State(index as u64).to_string(),
                        ValidatorState::from(state).to_string(),
                    );
                }
                (None, _) => storage.add(
                    ValidatorsQueryStorageKeys::State(index as u64).to_string(),
                    ValidatorState::Unknown.to_string(),
                ),
            }
            storage.add(
                ValidatorsQueryStorageKeys::Validator(index as u64).to_string(),
                address.to_string(),
            );
        }

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorsQueryParametersDto {
    pub epoch: Option<Value>,
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
            Value::Fuzz { .. } => unimplemented!(),
        });

        Self { epoch }
    }
}
