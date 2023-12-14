use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, core::types::token::Amount, signing::default_sign, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

pub enum TxInitAccountStorageKeys {
    SourceAddress,
    ValidatorAddress,
    Amount,
}

impl ToString for TxInitAccountStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitAccountStorageKeys::SourceAddress => "source-address".to_string(),
            TxInitAccountStorageKeys::ValidatorAddress => "validator-address".to_string(),
            TxInitAccountStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxBond {}

impl TxBond {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxBond {
    type P = TxBondParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let amount = Amount::from(parameters.amount);
        let validator_address = parameters.validator.to_namada_address(sdk).await;
        let source_public_key = parameters.source.to_public_key(sdk).await;

        let bond_tx_builder = sdk
            .namada
            .new_bond(validator_address.clone(), amount)
            .source(source_address.clone())
            .signing_keys(vec![source_public_key]);

        let (mut bond_tx, signing_data, _epoch) = bond_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build bond");

        sdk.namada
            .sign(
                &mut bond_tx,
                &bond_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign reveal bond");

        let tx = sdk.namada.submit(bond_tx, &bond_tx_builder.tx).await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(
            TxInitAccountStorageKeys::ValidatorAddress.to_string(),
            validator_address.to_string(),
        );
        storage.add(
            TxInitAccountStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxInitAccountStorageKeys::Amount.to_string(),
            amount.raw_amount().to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxBondParametersDto {
    source: Value,
    validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxBondParameters {
    source: AccountIndentifier,
    validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for TxBondParameters {
    type D = TxBondParametersDto;

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
            validator,
            amount,
        }
    }
}
