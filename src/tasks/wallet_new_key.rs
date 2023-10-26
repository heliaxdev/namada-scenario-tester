use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, StepStorage, Storage}, sdk::namada::Sdk,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct WalletNewKey {
    rpc: String,
    chain_id: String,
}

impl WalletNewKey {
    pub fn new(sdk: &Sdk) -> Self {
        Self {
            rpc: sdk.rpc.clone(),
            chain_id: sdk.chain_id.clone(),
        }
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

    async fn execute(&self,  sdk: &Sdk, _dto: Self::P, _state: &Storage) -> StepResult {
        let alias = self.generate_random_alias();
        println!(
            "namadaw address gen --alias {} --unsafe-dont-encrypt --node {}",
            alias, format!("{}/{}", self.rpc, self.chain_id)
        );

        let mut storage = StepStorage::default();
        storage.add("address-alias".to_string(), alias.to_string());
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

        let address = Address::from_alias(alias);

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
