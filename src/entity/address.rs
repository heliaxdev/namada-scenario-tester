use std::str::FromStr;

use namada_sdk::{
    address::Address,
    key::common::{self, PublicKey},
    rpc, Namada,
};

use crate::{sdk::namada::Sdk, state::state::StateAddress};

pub const ADDRESS_PREFIX: &str = namada_sdk::string_encoding::ADDRESS_HRP;

#[derive(Clone, Debug)]
pub enum AccountIndentifier {
    Alias(String),
    Address(String),
    PublicKey(String),
    StateAddress(StateAddress),
}

impl AccountIndentifier {
    pub async fn to_namada_address(&self, sdk: &Sdk) -> Address {
        match self {
            AccountIndentifier::Alias(alias) => match alias.to_lowercase().as_str() {
                "nam" => rpc::query_native_token(sdk.namada.client()).await.unwrap(),
                _ => {
                    let wallet = sdk.namada.wallet.read().await;
                    wallet.find_address(alias).unwrap().as_ref().clone()
                }
            },
            AccountIndentifier::PublicKey(_) => {
                panic!()
            }
            AccountIndentifier::Address(address) => Address::decode(address).unwrap(),
            AccountIndentifier::StateAddress(metadata) => {
                Address::decode(metadata.address.clone()).unwrap()
            }
        }
    }

    pub async fn to_secret_key(&self, sdk: &Sdk) -> common::SecretKey {
        // We match alias first in order to avoid a wallet lock issue
        let alias = match self {
            AccountIndentifier::Alias(alias) => alias.clone(),
            AccountIndentifier::Address(address) => {
                let address = Address::decode(address).unwrap();
                let wallet = sdk.namada.wallet.read().await;
                let alias = wallet.find_alias(&address).unwrap();
                alias.to_string()
            }
            AccountIndentifier::PublicKey(public_key) => {
                let public_key = PublicKey::from_str(public_key).unwrap();
                let mut wallet = sdk.namada.wallet.write().await;
                let sk = wallet.find_key_by_pk(&public_key, None).unwrap();
                return sk;
            }
            AccountIndentifier::StateAddress(_metadata) => unimplemented!(),
        };
        let mut wallet = sdk.namada.wallet.write().await;
        wallet.find_secret_key(&alias, None).unwrap()
    }

    pub async fn to_public_key(&self, sdk: &Sdk) -> common::PublicKey {
        // We match alias first in order to avoid a wallet lock issue
        let alias = match self {
            AccountIndentifier::Alias(alias) => alias.clone(),
            AccountIndentifier::Address(address) => {
                let address = Address::decode(address).unwrap();
                let wallet = sdk.namada.wallet.read().await;
                let alias = wallet.find_alias(&address).unwrap();
                alias.to_string()
            }
            AccountIndentifier::PublicKey(public_key) => {
                return PublicKey::from_str(public_key).unwrap()
            }
            AccountIndentifier::StateAddress(_metadata) => unimplemented!(),
        };
        let wallet = sdk.namada.wallet.read().await;
        wallet.find_public_key(&alias).unwrap()
    }
}
