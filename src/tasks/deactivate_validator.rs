use async_trait::async_trait;

use namada_sdk::{
    args::TxDeactivateValidator as SdkDeactivateValidatorTx, signing::default_sign, Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskParam};

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

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let deactivate_validator_tx_builder =
            sdk.namada.new_deactivate_validator(source_address.clone());

        let deactivate_validator_tx_builder = self
            .add_settings(sdk, deactivate_validator_tx_builder, settings)
            .await;

        let (mut deactivate_validator_tx, signing_data) = deactivate_validator_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

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

        let tx = sdk
            .namada
            .submit(
                deactivate_validator_tx.clone(),
                &deactivate_validator_tx_builder.tx,
            )
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&deactivate_validator_tx, &tx) {
            let errors = Self::get_tx_errors(&tx.unwrap()).unwrap_or_default();
            return StepResult::fail(errors);
        }

        storage.add(
            TxDeactivateValidatorStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        StepResult::success(storage)
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
