use async_trait::async_trait;
use namada_sdk::types::{
    address::Address,
    key::{RefTo, SchemeType},
};
use rand::{distributions::Alphanumeric, Rng};
use rand_core::OsRng;
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
};

use super::{Task, TaskParam};

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
    pub fn generate_random_alias(&self) -> String {
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

    async fn execute(&self, sdk: &Sdk, _dto: Self::P, _state: &Storage) -> StepResult {
        let alias = self.generate_random_alias();

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

#[derive(Clone, Debug, Deserialize, Default)]
pub struct WalletNewKeyParametersDto {}

#[derive(Clone, Debug)]
pub struct WalletNewKeyParameters {}

impl TaskParam for WalletNewKeyParameters {
    type D = WalletNewKeyParametersDto;

    fn from_dto(_dto: Self::D, _state: &Storage) -> Self {
        WalletNewKeyParameters {}
    }
}
