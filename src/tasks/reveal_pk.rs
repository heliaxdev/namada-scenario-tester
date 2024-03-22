use async_trait::async_trait;

use namada_sdk::{address::Address, args::TxBuilder, signing::default_sign, Namada};

use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

pub enum TxRevealPkStorageKeys {
    PublicKey,
    Address,
}

impl ToString for TxRevealPkStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxRevealPkStorageKeys::PublicKey => "public-key".to_string(),
            TxRevealPkStorageKeys::Address => "address".to_string(),
        }
    }
}

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
        let source_public_key = parameters.source.to_public_key(sdk).await;

        let reveal_pk_tx_builder = sdk
            .namada
            .new_reveal_pk(source_public_key.clone())
            .signing_keys(vec![source_public_key.clone()]);

        let (mut reveal_tx, signing_data) = reveal_pk_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");

        sdk.namada
            .sign(
                &mut reveal_tx,
                &reveal_pk_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign reveal pk tx");

        let tx = sdk.namada.submit(reveal_tx, &reveal_pk_tx_builder.tx).await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        let address = Address::from(&source_public_key);

        let mut storage = StepStorage::default();
        storage.add(
            TxRevealPkStorageKeys::Address.to_string(),
            address.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::PublicKey.to_string(),
            source_public_key.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]

pub struct RevealPkParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]

pub struct RevealPkParameters {
    source: AccountIndentifier,
}

impl TaskParam for RevealPkParameters {
    type D = RevealPkParametersDto;

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

        Self { source }
    }
}
