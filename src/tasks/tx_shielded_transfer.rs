use async_trait::async_trait;
use namada_sdk::rpc::TxResponse;
use namada_sdk::string_encoding::MASP_EXT_SPENDING_KEY_HRP;
use namada_sdk::tx::ProcessTxResponse;
use namada_sdk::{
    args::{InputAmount, TxShieldedTransfer as NamadaTxShieldedTransfer, TxShieldedTransferData},
    signing::default_sign,
    string_encoding::MASP_PAYMENT_ADDRESS_HRP,
    token::{self, DenominatedAmount},
    Namada,
};
use serde::{Deserialize, Serialize};

use crate::utils::settings::TxSettings;
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskError, TaskParam};

pub enum TxShieldedTransferStorageKeys {
    Source,
    Target,
    Amount,
    Token,
}

impl ToString for TxShieldedTransferStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxShieldedTransferStorageKeys::Source => "source".to_string(),
            TxShieldedTransferStorageKeys::Target => "target".to_string(),
            TxShieldedTransferStorageKeys::Amount => "amount".to_string(),
            TxShieldedTransferStorageKeys::Token => "token".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxShieldedTransfer {}

impl TxShieldedTransfer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxShieldedTransfer {
    type P = TxShieldedTransferParameters;
    type B = NamadaTxShieldedTransfer;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let source_address = parameters.source.to_spending_key(sdk).await;
        let target_address = parameters.target.to_payment_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let token_amount = token::Amount::from_u64(parameters.amount);
        let denominated_amount = DenominatedAmount::native(token_amount);

        let tx_transfer_data = TxShieldedTransferData {
            source: source_address.clone(),
            target: target_address.clone(),
            token: token_address.clone(),
            amount: InputAmount::Validated(denominated_amount),
        };

        let transfer_tx_builder = sdk
            .namada
            .new_shielded_transfer(vec![tx_transfer_data], vec![]);

        let mut transfer_tx_builder = self.add_settings(sdk, transfer_tx_builder, settings).await;

        let (mut transfer_tx, signing_data) = transfer_tx_builder
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
        let tx = sdk
            .namada
            .submit(transfer_tx.clone(), &transfer_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&transfer_tx, &tx) {
            let errors = Self::get_tx_errors(&transfer_tx, &tx.unwrap()).unwrap_or_default();
            return Ok(StepResult::fail(errors));
        }

        let Ok(ProcessTxResponse::Applied(TxResponse { height, .. })) = &tx else {
            unreachable!()
        };

        storage.add(
            TxShieldedTransferStorageKeys::Source.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxShieldedTransferStorageKeys::Target.to_string(),
            target_address.to_string(),
        );
        storage.add(
            TxShieldedTransferStorageKeys::Amount.to_string(),
            token_amount.raw_amount().to_string(),
        );
        storage.add(
            TxShieldedTransferStorageKeys::Token.to_string(),
            token_address.to_string(),
        );
        storage.add("stx-height".to_string(), height.to_string());

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxShieldedTransferParametersDto {
    pub source: Value,
    pub target: Value,
    pub amount: Value,
    pub token: Value,
}

#[derive(Clone, Debug)]
pub struct TxShieldedTransferParameters {
    source: AccountIndentifier,
    target: AccountIndentifier,
    amount: u64,
    token: AccountIndentifier,
}

impl TaskParam for TxShieldedTransferParameters {
    type D = TxShieldedTransferParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.source {
            Value::Ref { value, field } => {
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    _ => AccountIndentifier::SpendingKey(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(MASP_EXT_SPENDING_KEY_HRP) {
                    AccountIndentifier::SpendingKey(value)
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
