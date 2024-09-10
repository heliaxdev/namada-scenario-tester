use std::fmt::Display;

use async_trait::async_trait;

use namada_sdk::{
    args::TxDeactivateValidator as SdkDeactivateValidatorTx, signing::default_sign, Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxDeactivateValidatorStorageKeys {
    ValidatorAddress,
}

impl ToString for TxDeactivateValidatorStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxDeactivateValidatorStorageKeys::ValidatorAddress => "address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxDeactivateValidator {}

impl TxDeactivateValidator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxDeactivateValidator {
    type P = DeactivateValidatorParameters;
    type B = SdkDeactivateValidatorTx;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let deactivate_validator_tx_builder =
            sdk.namada.new_deactivate_validator(source_address.clone());

        let deactivate_validator_tx_builder = self
            .add_settings(sdk, deactivate_validator_tx_builder, settings)
            .await;

        let (mut deactivate_validator_tx, signing_data) = deactivate_validator_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut deactivate_validator_tx,
                &deactivate_validator_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxDeactivateValidatorStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        Ok(BuildResult::new(
            deactivate_validator_tx,
            deactivate_validator_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxDeactivateValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-deactivate-validator")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct DeactivateValidatorParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]

pub struct DeactivateValidatorParameters {
    source: AccountIndentifier,
}

impl TaskParam for DeactivateValidatorParameters {
    type D = DeactivateValidatorParametersDto;

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
