use async_trait::async_trait;

use namada_sdk::{
    args::{TxBecomeValidator as SdkBecomeValidatorTx, TxBuilder},
    dec::Dec,
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

use super::{Task, TaskParam};

pub enum TxBecomeValidatorStorageKeys {
    ValidatorAddress,
}

impl ToString for TxBecomeValidatorStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxBecomeValidatorStorageKeys::ValidatorAddress => "address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxBecomeValidator {}

impl TxBecomeValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl TxBecomeValidator {
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
impl Task for TxBecomeValidator {
    type P = BecomeValidatorParameters;
    type B = SdkBecomeValidatorTx;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let commission_rate = Dec::new(parameters.commission_rate as i128, 2).unwrap();

        let consensus_key_alias = self.generate_random_alias("consensus");
        let eth_cold_key_alias = self.generate_random_alias("eth-cold");
        let eth_hot_key_alias = self.generate_random_alias("eth-hot");
        let protocol_key = self.generate_random_alias("protocol");

        let mut wallet = sdk.namada.wallet.write().await;

        let consensus_pk = wallet
            .gen_store_secret_key(
                SchemeType::Ed25519,
                Some(consensus_key_alias.clone()),
                true,
                None,
                &mut OsRng,
            )
            .expect("Key generation should not fail.")
            .1
            .ref_to();

        let eth_cold_pk = wallet
            .gen_store_secret_key(
                SchemeType::Secp256k1,
                Some(eth_cold_key_alias.clone()),
                true,
                None,
                &mut OsRng,
            )
            .expect("Key generation should not fail.")
            .1
            .ref_to();

        let eth_hot_pk = wallet
            .gen_store_secret_key(
                SchemeType::Secp256k1,
                Some(eth_hot_key_alias.clone()),
                true,
                None,
                &mut OsRng,
            )
            .expect("Key generation should not fail.")
            .1
            .ref_to();

        let protocol_key = wallet
            .gen_store_secret_key(
                SchemeType::Ed25519,
                Some(protocol_key.clone()),
                true,
                None,
                &mut OsRng,
            )
            .expect("Key generation should not fail.")
            .1
            .ref_to();

        wallet.save().expect("unable to save wallet");

        drop(wallet);

        let become_validator_tx_builder = sdk.namada.new_become_validator(
            source_address.clone(),
            commission_rate,
            Dec::one(),
            consensus_pk,
            eth_cold_pk,
            eth_hot_pk,
            protocol_key,
            "gianmarco+scenario-tester@heliax.dev".to_string(),
        );

        let become_validator_tx_builder = self
            .add_settings(sdk, become_validator_tx_builder, settings)
            .await;

        let (mut become_validator_tx, signing_data) = become_validator_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

        sdk.namada
            .sign(
                &mut become_validator_tx,
                &become_validator_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let tx = sdk
            .namada
            .submit(become_validator_tx.clone(), &become_validator_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&become_validator_tx, &tx) {
            let errors = Self::get_tx_errors(&tx.unwrap()).unwrap_or_default();
            return StepResult::fail(errors);
        }

        storage.add(
            TxBecomeValidatorStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct BecomeValidatorParametersDto {
    pub source: Value,
    pub commission_rate: Value,
}

#[derive(Clone, Debug)]

pub struct BecomeValidatorParameters {
    source: AccountIndentifier,
    commission_rate: u64,
}

impl TaskParam for BecomeValidatorParameters {
    type D = BecomeValidatorParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
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

        let commission_rate = match dto.commission_rate {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => rand::thread_rng().gen_range(1..100),
        };

        Self {
            source,
            commission_rate,
        }
    }
}
