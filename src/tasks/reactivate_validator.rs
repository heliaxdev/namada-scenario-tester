use std::fmt::Display;

use async_trait::async_trait;

use namada_sdk::{
    args::TxReactivateValidator as SdkReactivateValidatorTx, signing::default_sign, Namada,
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxReactivateValidatorStorageKeys {
    ValidatorAddress,
}

impl ToString for TxReactivateValidatorStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxReactivateValidatorStorageKeys::ValidatorAddress => "address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxReactivateValidator {}

impl TxReactivateValidator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxReactivateValidator {
    type P = ReactivateValidatorParameters;
    type B = SdkReactivateValidatorTx;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let reactivate_validator_tx_builder =
            sdk.namada.new_reactivate_validator(source_address.clone());

        let reactivate_validator_tx_builder = self
            .add_settings(sdk, reactivate_validator_tx_builder, settings)
            .await;

        let (mut reactivate_validator_tx, signing_data) = reactivate_validator_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut reactivate_validator_tx,
                &reactivate_validator_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxReactivateValidatorStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        Ok(BuildResult::new(
            reactivate_validator_tx,
            reactivate_validator_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxReactivateValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-reactivate-validator")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct ReactivateValidatorParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]

pub struct ReactivateValidatorParameters {
    source: AccountIndentifier,
}

impl TaskParam for ReactivateValidatorParameters {
    type D = ReactivateValidatorParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.source {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
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

        Some(Self { source })
    }
}
