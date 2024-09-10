use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::{
    args::{InputAmount, TxTransparentTransferData},
    signing::default_sign,
    token::{self, DenominatedAmount},
    Namada,
};
use serde::{Deserialize, Serialize};

use crate::utils::settings::TxSettings;
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxTransparentTransferStorageKeys {
    Source,
    Target,
    Amount,
    Token,
}

impl ToString for TxTransparentTransferStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxTransparentTransferStorageKeys::Source => "source".to_string(),
            TxTransparentTransferStorageKeys::Target => "target".to_string(),
            TxTransparentTransferStorageKeys::Amount => "amount".to_string(),
            TxTransparentTransferStorageKeys::Token => "token".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxTransparentTransfer {}

impl TxTransparentTransfer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxTransparentTransfer {
    type P = TxTransparentTransferParameters;
    type B = namada_sdk::args::TxTransparentTransfer;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let target_address = parameters.target.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let token_amount = token::Amount::from_u64(parameters.amount);

        let tx_transfer_data = TxTransparentTransferData {
            source: source_address.clone(),
            target: target_address.clone(),
            token: token_address.clone(),
            amount: InputAmount::Unvalidated(DenominatedAmount::native(token_amount)),
        };

        let transfer_tx_builder = sdk.namada.new_transparent_transfer(vec![tx_transfer_data]);

        let mut transfer_tx_builder = self.add_settings(sdk, transfer_tx_builder, settings).await;

        let (mut transfer_tx, signing_data) = transfer_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut transfer_tx,
                &transfer_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxTransparentTransferStorageKeys::Source.to_string(),
            source_address.to_string(),
        );
        step_storage.add(
            TxTransparentTransferStorageKeys::Target.to_string(),
            target_address.to_string(),
        );
        step_storage.add(
            TxTransparentTransferStorageKeys::Amount.to_string(),
            token_amount.raw_amount().to_string(),
        );
        step_storage.add(
            TxTransparentTransferStorageKeys::Token.to_string(),
            token_address.to_string(),
        );

        Ok(BuildResult::new(
            transfer_tx,
            transfer_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxTransparentTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-transparent-transfer")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxTransparentTransferParametersDto {
    pub source: Value,
    pub target: Value,
    pub amount: Value,
    pub token: Value,
}

#[derive(Clone, Debug)]
pub struct TxTransparentTransferParameters {
    source: AccountIndentifier,
    target: AccountIndentifier,
    amount: u64,
    token: AccountIndentifier,
}

impl TaskParam for TxTransparentTransferParameters {
    type D = TxTransparentTransferParametersDto;

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
        let target = match dto.target {
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
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                state.get_step_item(&value, &field).parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };
        let token = match dto.token {
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

        Some(Self {
            source,
            target,
            amount,
            token,
        })
    }
}
