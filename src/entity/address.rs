use namada_sdk::core::types::address::Address;

use crate::{sdk::namada::Sdk, state::state::StateAddress};

pub const ADDRESS_PREFIX: &str = namada_sdk::core::types::string_encoding::ADDRESS_HRP;

#[derive(Clone, Debug)]
pub enum AccountIndentifier {
    Alias(String),
    Address(String),
    StateAddress(StateAddress),
}

impl AccountIndentifier {
    pub async fn to_namada_address(&self, sdk: &Sdk) -> Address {
        match self {
            AccountIndentifier::Alias(alias) => {
                let wallet = sdk.namada.wallet.write().await;
                wallet.find_address(alias).unwrap().as_ref().clone()
            }
            AccountIndentifier::Address(address) => Address::decode(address).unwrap(),
            AccountIndentifier::StateAddress(metadata) => {
                Address::decode(metadata.address.clone()).unwrap()
            }
        }
    }
}
