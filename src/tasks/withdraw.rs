use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, signing::default_sign, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

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

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let validator_address = parameters.validator.to_namada_address(sdk).await;
        let source_public_key = parameters.source.to_public_key(sdk).await;

        let withdraw_tx_builder = sdk
            .namada
            .new_withdraw(validator_address.clone())
            .source(source_address.clone())
            .signing_keys(vec![source_public_key]);

        let (mut withdraw_tx, signing_data) = withdraw_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build withdraw");

        sdk.namada
            .sign(
                &mut withdraw_tx,
                &withdraw_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign unbond");

        let tx = sdk
            .namada
            .submit(withdraw_tx, &withdraw_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(
            TxWithdrawStorageKeys::ValidatorAddress.to_string(),
            validator_address.to_string(),
        );
        storage.add(
            TxWithdrawStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
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
        let validator = match dto.validator {
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

        Self { source, validator }
    }
}
