use async_trait::async_trait;

use namada_sdk::{
    address::Address,
    args::{RevealPk, TxBuilder},
    signing::default_sign,
    Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

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
    type B = RevealPk;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        _settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let source_public_key = parameters.source.to_public_key(sdk).await;
        let faucet_public_key = AccountIndentifier::Alias("faucet".to_string())
            .to_public_key(sdk)
            .await;

        let reveal_pk_tx_builder = sdk
            .namada
            .new_reveal_pk(source_public_key.clone())
            .signing_keys(vec![source_public_key.clone()])
            .wrapper_fee_payer(faucet_public_key); // workaround due to scenario generator limitation

        let (mut reveal_tx, signing_data) = reveal_pk_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut reveal_tx,
                &reveal_pk_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let tx = sdk
            .namada
            .submit(reveal_tx.clone(), &reveal_pk_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&reveal_tx, &tx) {
            let errors = Self::get_tx_errors(&reveal_tx, &tx.unwrap()).unwrap_or_default();
            return Ok(StepResult::fail(errors));
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

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct RevealPkParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]

pub struct RevealPkParameters {
    source: AccountIndentifier,
}

impl TaskParam for RevealPkParameters {
    type D = RevealPkParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.source {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
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

        Some(Self { source })
    }
}
