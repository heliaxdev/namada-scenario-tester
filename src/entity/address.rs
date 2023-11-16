use namada_sdk::core::types::{address::Address, key::common};

use crate::{sdk::namada::Sdk, state::state::StateAddress};

pub const ADDRESS_PREFIX: &str = "tnam";

#[derive(Clone, Debug)]
pub enum AccountIndentifier {
    Alias(String),
    Address(String),
    StateAddress(StateAddress),
}

impl AccountIndentifier {
    pub async fn to_namada_address(&self, sdk: &Sdk<'_>) -> Address {
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
    pub async fn to_secret_key(&self, sdk: &Sdk<'_>) -> common::SecretKey {
        println!("Trying to get secret key");
        match self {
            AccountIndentifier::Alias(alias) => {
                let mut wallet = sdk.namada.wallet.write().await;
                wallet.find_secret_key(alias, None).unwrap().clone()
            }
            AccountIndentifier::Address(address) => {
                let address = Address::decode(address).unwrap();
                println!("decoded address");
                let wallet_tmp = sdk.namada.wallet.read().await;
                let alias = wallet_tmp.find_alias(&address).unwrap();
                let mut wallet = sdk.namada.wallet.write().await;
                let source_secret_key = wallet.find_secret_key(alias, None).unwrap().clone();
                drop(wallet);
                source_secret_key
            }
            AccountIndentifier::StateAddress(metadata) => unimplemented!()
        }
    }
}
