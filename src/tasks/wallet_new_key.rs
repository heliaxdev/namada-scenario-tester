use async_trait::async_trait;
use namada_sdk::args::Bond;
use namada_sdk::{address::Address, key::SchemeType};
use rand::rngs::OsRng;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::{Task, TaskParam};
use crate::utils::settings::TxSettings;
use crate::utils::value::Value;
use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
};
use namada_sdk::key::RefTo;

pub enum WalletNewKeyStorageKeys {
    Alias,
    PrivateKey,
    PublicKey,
    Address,
}

impl ToString for WalletNewKeyStorageKeys {
    fn to_string(&self) -> String {
        match self {
            WalletNewKeyStorageKeys::Address => "address".to_string(),
            WalletNewKeyStorageKeys::Alias => "alias".to_string(),
            WalletNewKeyStorageKeys::PublicKey => "public-key".to_string(),
            WalletNewKeyStorageKeys::PrivateKey => "private-key".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WalletNewKey {}

impl WalletNewKey {
    pub fn new() -> Self {
        Self {}
    }
}

impl WalletNewKey {
    pub fn generate_random_alias() -> String {
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        format!("lt-addr-{}", random_suffix)
    }
}

#[async_trait(?Send)]
impl Task for WalletNewKey {
    type P = WalletNewKeyParameters;
    type B = Bond; // just a placeholder

    async fn execute(
        &self,
        sdk: &Sdk,
        dto: Self::P,
        _settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let alias = dto.alias;

        let mut wallet = sdk.namada.wallet.write().await;

        let keypair =
            wallet.gen_store_secret_key(SchemeType::Ed25519, Some(alias), true, None, &mut OsRng);

        let (alias, sk) = if let Some((alias, sk)) = keypair {
            wallet.save().expect("unable to save wallet");
            (alias, sk)
        } else {
            return StepResult::fail();
        };

        let address = Address::from(&sk.ref_to()).to_string();

        let mut storage = StepStorage::default();
        storage.add(
            WalletNewKeyStorageKeys::Alias.to_string(),
            alias.to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::PublicKey.to_string(),
            sk.ref_to().to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::Address.to_string(),
            address.clone(),
        );
        storage.add(
            WalletNewKeyStorageKeys::PrivateKey.to_string(),
            sk.to_string(),
        );

        let address = StateAddress::new_implicit(alias, address);

        StepResult::success_with_accounts(storage, vec![address])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletNewKeyParametersDto {
    pub alias: Value,
}

#[derive(Clone, Debug)]
pub struct WalletNewKeyParameters {
    alias: String,
}

impl TaskParam for WalletNewKeyParameters {
    type D = WalletNewKeyParametersDto;

    fn parameter_from_dto(dto: Self::D, _state: &Storage) -> Self {
        let alias = match dto.alias {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.to_string(),
            Value::Fuzz { .. } => WalletNewKey::generate_random_alias(),
        };

        WalletNewKeyParameters { alias }
    }
}
