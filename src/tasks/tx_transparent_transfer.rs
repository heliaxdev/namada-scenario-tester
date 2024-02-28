use async_trait::async_trait;
// TODO: this used to be pub in SDK
use namada_core::masp::{TransferSource, TransferTarget};
use namada_sdk::{
    args::InputAmount,
    signing::default_sign,
    token::{self, DenominatedAmount},
    Namada,
};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

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

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let target_address = parameters.target.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let token_amount = token::Amount::from_u64(parameters.amount);

        let mut transfer_tx_builder = sdk.namada.new_transfer(
            TransferSource::Address(source_address.clone()),
            TransferTarget::Address(target_address.clone()),
            token_address.clone(),
            InputAmount::Unvalidated(DenominatedAmount::native(token_amount)),
        );

        let (mut transfer_tx, signing_data, _epoch) = transfer_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");

        sdk.namada
            .sign(
                &mut transfer_tx,
                &transfer_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign reveal pk tx");
        let tx = sdk
            .namada
            .submit(transfer_tx, &transfer_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(
            TxTransparentTransferStorageKeys::Source.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxTransparentTransferStorageKeys::Target.to_string(),
            target_address.to_string(),
        );
        storage.add(
            TxTransparentTransferStorageKeys::Amount.to_string(),
            token_amount.raw_amount().to_string(),
        );
        storage.add(
            TxTransparentTransferStorageKeys::Token.to_string(),
            token_address.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxTransparentTransferParametersDto {
    source: Value,
    target: Value,
    amount: Value,
    token: Value,
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

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
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
            Value::Fuzz {} => unimplemented!(),
        };
        let target = match dto.target {
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
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                state.get_step_item(&value, &field).parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
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
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            source,
            target,
            amount,
            token,
        }
    }
}
