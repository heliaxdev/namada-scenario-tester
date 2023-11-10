use async_trait::async_trait;

use namada_sdk::{args::TxBuilder, core::types::address::Address, Namada};

use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]

pub struct TxRevealPk {}

impl TxRevealPk {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]

impl Task for TxRevealPk {
    type P = RevealPkParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let alias = match parameters.source {
            AccountIndentifier::Alias(alias) => alias,
            AccountIndentifier::Address(_) => panic!(),
            AccountIndentifier::StateAddress(state) => state.alias,
        };
        let source_secret_key = sdk.find_secret_key(&alias).await;
        let source_public_key = source_secret_key.to_public();

        let reveal_pk_tx_builder = sdk
            .namada
            .new_reveal_pk(source_public_key.clone())
            .signing_keys(vec![source_secret_key]);

        let (mut reveal_tx, signing_data, _epoch) = reveal_pk_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");

        sdk.namada
            .sign(&mut reveal_tx, &reveal_pk_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign reveal pk tx");

        let _tx = sdk.namada.submit(reveal_tx, &reveal_pk_tx_builder.tx).await;

        let address = Address::from(&source_public_key);

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), alias);
        storage.add("address".to_string(), address.to_string());

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]

pub struct RevealPkParametersDto {
    source: Value,
}

#[derive(Clone, Debug)]

pub struct RevealPkParameters {
    source: AccountIndentifier,
}

impl TaskParam for RevealPkParameters {
    type D = RevealPkParametersDto;

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

        Self { source }
    }
}