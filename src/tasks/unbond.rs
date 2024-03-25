use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, signing::default_sign, token::Amount, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

pub enum TxUnbondStorageKeys {
    SourceAddress,
    ValidatorAddress,
    Amount,
}

impl ToString for TxUnbondStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxUnbondStorageKeys::SourceAddress => "source-address".to_string(),
            TxUnbondStorageKeys::ValidatorAddress => "validator-address".to_string(),
            TxUnbondStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxUnbond {}

impl TxUnbond {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxUnbond {
    type P = TxUnbondParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let amount = Amount::from(parameters.amount);
        let validator_address = parameters.validator.to_namada_address(sdk).await;
        let source_public_key = parameters.source.to_public_key(sdk).await;

        let unbond_tx_builder = sdk
            .namada
            .new_unbond(validator_address.clone(), amount)
            .source(source_address.clone())
            .force(true)
            .signing_keys(vec![source_public_key]);

        let (mut unbond_tx, signing_data, _) = unbond_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build bond");

        sdk.namada
            .sign(
                &mut unbond_tx,
                &unbond_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign unbond");

        let tx = sdk.namada.submit(unbond_tx, &unbond_tx_builder.tx).await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(
            TxUnbondStorageKeys::ValidatorAddress.to_string(),
            validator_address.to_string(),
        );
        storage.add(
            TxUnbondStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxUnbondStorageKeys::Amount.to_string(),
            amount.raw_amount().to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxUnbondParametersDto {
    pub source: Value,
    pub validator: Value,
    pub amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxUnbondParameters {
    source: AccountIndentifier,
    validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for TxUnbondParameters {
    type D = TxUnbondParametersDto;

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
            Value::Fuzz { .. } => unimplemented!(),
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
            Value::Fuzz { .. } => unimplemented!(),
        };
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                let amount = state.get_step_item(&value, &field);
                amount.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Self {
            source,
            validator,
            amount,
        }
    }
}
