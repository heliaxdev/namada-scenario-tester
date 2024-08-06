use async_trait::async_trait;
use namada_sdk::{
    args::{
        InputAmount, TxUnshieldingTransfer as NamadaTxUnshieldingTransfer,
        TxUnshieldingTransferData,
    },
    args::{NamadaTypes, SdkTypes, Tx, TxBuilder, TxExpiration},
    signing::default_sign,
    string_encoding::MASP_EXT_SPENDING_KEY_HRP,
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

pub struct UnshieldingTransferBuilder<C = SdkTypes>(NamadaTxUnshieldingTransfer<C>)
where
    C: NamadaTypes;

impl<C: NamadaTypes> TxBuilder<C> for UnshieldingTransferBuilder<C> {
    fn tx<F>(self, func: F) -> Self
    where
        F: FnOnce(Tx<C>) -> Tx<C>,
    {
        Self(NamadaTxUnshieldingTransfer {
            tx: func(self.0.tx),
            ..self.0
        })
    }
}

pub enum TxUnshieldingTransferStorageKeys {
    Source,
    Target,
    Amount,
    Token,
}

impl ToString for TxUnshieldingTransferStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxUnshieldingTransferStorageKeys::Source => "source".to_string(),
            TxUnshieldingTransferStorageKeys::Target => "target".to_string(),
            TxUnshieldingTransferStorageKeys::Amount => "amount".to_string(),
            TxUnshieldingTransferStorageKeys::Token => "token".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxUnshieldingTransfer {}

impl TxUnshieldingTransfer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxUnshieldingTransfer {
    type P = TxUnshieldingTransferParameters;
    type B = UnshieldingTransferBuilder;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let source_address = parameters.target.to_spending_key(sdk).await;
        let target_address = parameters.source.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let token_amount = token::Amount::from_u64(parameters.amount);
        let denominated_amount = DenominatedAmount::native(token_amount);

        let tx_transfer_data = TxUnshieldingTransferData {
            target: target_address.clone(),
            token: token_address.clone(),
            amount: InputAmount::Validated(denominated_amount),
        };

        let mut transfer_tx_builder = UnshieldingTransferBuilder(
            sdk.namada
                .new_unshielding_transfer(source_address, vec![tx_transfer_data], vec![]),
        );

        transfer_tx_builder.0.tx.signing_keys = vec![
            settings
                .gas_payer
                .as_ref()
                .ok_or_else(|| TaskError::Build("No gas payer was present".into()))?
                .to_public_key(sdk)
                .await,
        ];
        transfer_tx_builder.0.tx.expiration = TxExpiration::NoExpiration;

        let UnshieldingTransferBuilder(mut transfer_tx_builder) =
            self.add_settings(sdk, transfer_tx_builder, settings).await;

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

        storage.add(
            TxUnshieldingTransferStorageKeys::Source.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxUnshieldingTransferStorageKeys::Target.to_string(),
            target_address.to_string(),
        );
        storage.add(
            TxUnshieldingTransferStorageKeys::Amount.to_string(),
            token_amount.raw_amount().to_string(),
        );
        storage.add(
            TxUnshieldingTransferStorageKeys::Token.to_string(),
            token_address.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxUnshieldingTransferParametersDto {
    pub source: Value,
    pub target: Value,
    pub amount: Value,
    pub token: Value,
}

#[derive(Clone, Debug)]
pub struct TxUnshieldingTransferParameters {
    source: AccountIndentifier,
    target: AccountIndentifier,
    amount: u64,
    token: AccountIndentifier,
}

impl TaskParam for TxUnshieldingTransferParameters {
    type D = TxUnshieldingTransferParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.target {
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
        let target = match dto.source {
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
