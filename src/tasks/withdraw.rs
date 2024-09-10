use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::{args::Withdraw, signing::default_sign, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxWithdrawStorageKeys {
    SourceAddress,
    ValidatorAddress,
}

impl ToString for TxWithdrawStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxWithdrawStorageKeys::SourceAddress => "source-address".to_string(),
            TxWithdrawStorageKeys::ValidatorAddress => "validator-address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxWithdraw {}

impl TxWithdraw {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxWithdraw {
    type P = TxWithdrawParameters;
    type B = Withdraw;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let validator_address = parameters.validator.to_namada_address(sdk).await;

        let withdraw_tx_builder = sdk
            .namada
            .new_withdraw(validator_address.clone())
            .source(source_address.clone());

        let withdraw_tx_builder = self.add_settings(sdk, withdraw_tx_builder, settings).await;

        let (mut withdraw_tx, signing_data) = withdraw_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut withdraw_tx,
                &withdraw_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxWithdrawStorageKeys::ValidatorAddress.to_string(),
            validator_address.to_string(),
        );
        step_storage.add(
            TxWithdrawStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );

        Ok(BuildResult::new(
            withdraw_tx,
            withdraw_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxWithdraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-withdraw")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxWithdrawParametersDto {
    pub source: Value,
    pub validator: Value,
}

#[derive(Clone, Debug)]
pub struct TxWithdrawParameters {
    source: AccountIndentifier,
    validator: AccountIndentifier,
}

impl TaskParam for TxWithdrawParameters {
    type D = TxWithdrawParametersDto;

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
        let validator = match dto.validator {
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

        Some(Self { source, validator })
    }
}
