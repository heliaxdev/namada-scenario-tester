use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, core::types::token::Amount, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};
use namada_sdk::signing::default_sign;

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct Bond {}

impl Bond {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for Bond {
    type P = BondParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let amount = Amount::from(parameters.amount);

        let alias = match parameters.source {
            AccountIndentifier::Alias(alias) => alias,
            AccountIndentifier::Address(_) => panic!(),
            AccountIndentifier::StateAddress(state) => state.alias,
        };

        let validator_address = parameters.validator.to_namada_address(sdk).await;
        let source_secret_key = sdk.find_secret_key(&alias).await;

        let bond_tx_builder = sdk
            .namada
            .new_bond(validator_address.clone(), amount)
            .source(source_address.clone())
            .signing_keys(vec![source_secret_key]);
        let (mut bond_tx, signing_data, _epoch) = bond_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build bond");
        sdk.namada
            .sign(&mut bond_tx, &bond_tx_builder.tx, signing_data, default_sign)
            .await
            .expect("unable to sign reveal bond");
        let _tx = sdk.namada.submit(bond_tx, &bond_tx_builder.tx).await;

        let mut storage = StepStorage::default();
        storage.add("address-validator".to_string(), source_address.to_string());
        storage.add("address-delegator".to_string(), source_address.to_string());
        storage.add("amount".to_string(), amount.to_string_native());

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BondParametersDto {
    source: Value,
    validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct BondParameters {
    source: AccountIndentifier,
    validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for BondParameters {
    type D = BondParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::StateAddress(state.get_address(&alias))
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
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::StateAddress(state.get_address(&alias))
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
            Value::Ref { value } => {
                let amount = state.get_step_item(&value, "amount");
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
