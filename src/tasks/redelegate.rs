use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, types::token::Amount, signing::default_sign, Namada};
use serde::Deserialize;

use super::{Task, TaskParam};
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

pub enum TxRevealPkStorageKeys {
    SourceValidatorAddress,
    DestValidatorAddress,
    SourceAddress,
    Amount,
}

impl ToString for TxRevealPkStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxRevealPkStorageKeys::SourceValidatorAddress => "source-validator-address".to_string(),
            TxRevealPkStorageKeys::DestValidatorAddress => "dest-validator-address".to_string(),
            TxRevealPkStorageKeys::SourceAddress => "source-address".to_string(),
            TxRevealPkStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxRedelegate {}

impl TxRedelegate {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxRedelegate {
    type P = TxRedelegateParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let source_address = parameters.source.to_namada_address(sdk).await;
        let source_public_key = parameters.source.to_public_key(sdk).await;
        let validator_src = parameters.src_validator.to_namada_address(sdk).await;
        let validator_target = parameters.dest_validator.to_namada_address(sdk).await;

        let bond_amount = Amount::from(parameters.amount);

        let redelegate_tx_builder = sdk
            .namada
            .new_redelegation(
                source_address.clone(),
                validator_src.clone(),
                validator_target.clone(),
                bond_amount,
            )
            .signing_keys(vec![source_public_key]);

        let (mut redelegate_tx, signing_data) = redelegate_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build redelegate tx");

        sdk.namada
            .sign(
                &mut redelegate_tx,
                &redelegate_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign redelegate tx");
        let tx = sdk
            .namada
            .submit(redelegate_tx, &redelegate_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(
            TxRevealPkStorageKeys::SourceValidatorAddress.to_string(),
            validator_src.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::DestValidatorAddress.to_string(),
            validator_target.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::Amount.to_string(),
            bond_amount.to_string_native(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxRedelegateParametersDto {
    source: Value,
    src_validator: Value,
    dest_validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxRedelegateParameters {
    source: AccountIndentifier,
    src_validator: AccountIndentifier,
    dest_validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for TxRedelegateParameters {
    type D = TxRedelegateParametersDto;

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
        let src_validator = match dto.src_validator {
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
        let dest_validator = match dto.dest_validator {
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
                let amount = state.get_step_item(&value, &field);
                amount.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            source,
            src_validator,
            dest_validator,
            amount,
        }
    }
}
