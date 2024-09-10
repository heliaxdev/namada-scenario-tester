use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::{
    args::{
        InputAmount, TxShieldingTransfer as NamadaTxShieldingTransfer, TxShieldingTransferData,
    },
    signing::default_sign,
    string_encoding::MASP_PAYMENT_ADDRESS_HRP,
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

pub enum TxShieldingTransferStorageKeys {
    Source,
    Target,
    Amount,
    Token,
}

impl ToString for TxShieldingTransferStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxShieldingTransferStorageKeys::Source => "source".to_string(),
            TxShieldingTransferStorageKeys::Target => "target".to_string(),
            TxShieldingTransferStorageKeys::Amount => "amount".to_string(),
            TxShieldingTransferStorageKeys::Token => "token".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxShieldingTransfer {}

impl TxShieldingTransfer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxShieldingTransfer {
    type P = TxShieldingTransferParameters;
    type B = NamadaTxShieldingTransfer;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let target_address = parameters.target.to_payment_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let token_amount = token::Amount::from_u64(parameters.amount);
        let denominated_amount = DenominatedAmount::native(token_amount);

        let tx_transfer_data = TxShieldingTransferData {
            source: source_address.clone(),
            token: token_address.clone(),
            amount: InputAmount::Validated(denominated_amount),
        };

        let transfer_tx_builder = sdk
            .namada
            .new_shielding_transfer(target_address, vec![tx_transfer_data]);

        let mut transfer_tx_builder = self.add_settings(sdk, transfer_tx_builder, settings).await;

        let (mut transfer_tx, signing_data, _epoch) = transfer_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|err| TaskError::Build(err.to_string()))?;

        sdk.namada
            .sign(
                &mut transfer_tx,
                &transfer_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .map_err(|err| TaskError::Build(err.to_string()))?;

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxShieldingTransferStorageKeys::Source.to_string(),
            source_address.to_string(),
        );
        step_storage.add(
            TxShieldingTransferStorageKeys::Target.to_string(),
            target_address.to_string(),
        );
        step_storage.add(
            TxShieldingTransferStorageKeys::Amount.to_string(),
            token_amount.raw_amount().to_string(),
        );
        step_storage.add(
            TxShieldingTransferStorageKeys::Token.to_string(),
            token_address.to_string(),
        );

        Ok(BuildResult::new(
            transfer_tx,
            transfer_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxShieldingTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-shielding-transfer")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxShieldingTransferParametersDto {
    pub source: Value,
    pub target: Value,
    pub amount: Value,
    pub token: Value,
}

#[derive(Clone, Debug)]
pub struct TxShieldingTransferParameters {
    source: AccountIndentifier,
    target: AccountIndentifier,
    amount: u64,
    token: AccountIndentifier,
}

impl TaskParam for TxShieldingTransferParameters {
    type D = TxShieldingTransferParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
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
        let target = match dto.target {
            Value::Ref { value, field } => {
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    _ => AccountIndentifier::PaymentAddress(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(MASP_PAYMENT_ADDRESS_HRP) {
                    AccountIndentifier::PaymentAddress(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                state.get_step_item(&value, &field).parse::<u64>().ok()?
            }
            Value::Value { value } => value.parse::<u64>().ok()?,
            Value::Fuzz { .. } => unimplemented!(),
        };
        let token = match dto.token {
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

        Some(Self {
            source,
            target,
            amount,
            token,
        })
    }
}
