use async_trait::async_trait;

use namada_sdk::{
    args::ConsensusKeyChange,
    key::{RefTo, SchemeType},
    signing::default_sign,
    Namada,
};

use rand::{distributions::Alphanumeric, Rng};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

pub enum TxChangeConsensusKeyStorageKeys {
    ValidatorAlias,
    ValidatorAddress,
    ConsensusPrivateKey,
    ConsensusPublicKey,
}

impl ToString for TxChangeConsensusKeyStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxChangeConsensusKeyStorageKeys::ValidatorAlias => "validator-alias".to_string(),
            TxChangeConsensusKeyStorageKeys::ValidatorAddress => "address".to_string(),
            TxChangeConsensusKeyStorageKeys::ConsensusPrivateKey => "private-key".to_string(),
            TxChangeConsensusKeyStorageKeys::ConsensusPublicKey => "public-key".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxChangeConsensusKey {}

impl TxChangeConsensusKey {
    pub fn new() -> Self {
        Self {}
    }
}

impl TxChangeConsensusKey {
    pub fn generate_random_alias(&self, namespace: &str) -> String {
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        format!("lt-addr-{}-{}", namespace, random_suffix)
    }
}

#[async_trait(?Send)]
impl Task for TxChangeConsensusKey {
    type P = TxChangeConsensusKeyParameters;
    type B = ConsensusKeyChange;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let consensus_key_alias = self.generate_random_alias("consensus");

        let mut wallet = sdk.namada.wallet.write().await;

        let consensus_sk = wallet
            .gen_store_secret_key(
                SchemeType::Ed25519,
                Some(consensus_key_alias.clone()),
                true,
                None,
                &mut OsRng,
            )
            .expect("Key generation should not fail.");

        let consensus_pk = consensus_sk.1.ref_to();

        wallet.save().expect("unable to save wallet");

        drop(wallet);

        let change_consensus_key_tx_builder = sdk
            .namada
            .new_change_consensus_key(source_address.clone(), consensus_pk.clone());
        let change_consensus_key_tx_builder = self
            .add_settings(sdk, change_consensus_key_tx_builder, settings)
            .await;

        let (mut change_consensus_key_tx, signing_data) = change_consensus_key_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut change_consensus_key_tx,
                &change_consensus_key_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        let tx = sdk
            .namada
            .submit(
                change_consensus_key_tx.clone(),
                &change_consensus_key_tx_builder.tx,
            )
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&change_consensus_key_tx, &tx) {
            match tx {
                Ok(tx) => {
                    let errors =
                        Self::get_tx_errors(&change_consensus_key_tx, &tx).unwrap_or_default();
                    return Ok(StepResult::fail(errors));
                }
                Err(e) => {
                    return Ok(StepResult::fail(e.to_string()));
                }
            }
        }

        storage.add(
            TxChangeConsensusKeyStorageKeys::ValidatorAlias.to_string(),
            consensus_key_alias.to_string(),
        );
        storage.add(
            TxChangeConsensusKeyStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxChangeConsensusKeyStorageKeys::ConsensusPrivateKey.to_string(),
            consensus_sk.1.to_string(),
        );
        storage.add(
            TxChangeConsensusKeyStorageKeys::ConsensusPublicKey.to_string(),
            consensus_pk.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct TxChangeConsensusKeyParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]

pub struct TxChangeConsensusKeyParameters {
    source: AccountIndentifier,
}

impl TaskParam for TxChangeConsensusKeyParameters {
    type D = TxChangeConsensusKeyParametersDto;

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
