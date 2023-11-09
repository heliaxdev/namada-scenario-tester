use async_trait::async_trait;
use namada_sdk::core::types::{
    address::Address,
    key::{RefTo, SchemeType},
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
};

use super::{Task, TaskParam};

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

        let keypair = wallet.gen_key(SchemeType::Ed25519, Some(alias), true, None, None, None);

        let (alias, sk) = if let Ok((alias, sk, _)) = keypair {
            wallet.save().expect("unable to save wallet");
            (alias, sk)
        } else {
            return StepResult::fail();
        };

        let address = Address::from(&sk.ref_to()).to_string();

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), alias.to_string());
        storage.add("address-pk".to_string(), sk.ref_to().to_string());
        storage.add("address".to_string(), address.clone());

        let address = StateAddress::new_implicit(alias, address);

        StepResult::success_with_accounts(storage, vec![address])
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalletNewKeyParametersDto {}

impl WalletNewKeyParametersDto {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
pub struct WalletNewKeyParameters {}

impl TaskParam for WalletNewKeyParameters {
    type D = WalletNewKeyParametersDto;

    fn from_dto(_dto: Self::D, _state: &Storage) -> Self {
        WalletNewKeyParameters {}
    }
}
