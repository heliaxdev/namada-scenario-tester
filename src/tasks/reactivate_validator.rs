use async_trait::async_trait;

use namada_sdk::{
    args::TxReactivateValidator as SdkReactivateValidatorTx, signing::default_sign, Namada,
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

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

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
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

        let tx = sdk
            .namada
            .submit(
                reactivate_validator_tx.clone(),
                &reactivate_validator_tx_builder.tx,
            )
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&reactivate_validator_tx, &tx) {
            let errors =
                Self::get_tx_errors(&reactivate_validator_tx, &tx.unwrap()).unwrap_or_default();
            return Ok(StepResult::fail(errors));
        }

        storage.add(
            TxReactivateValidatorStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        Ok(StepResult::success(storage))
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

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
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

        Self { source }
    }
}
