use std::str::FromStr;

use namada_sdk::{
    address::Address,
    key::common::{self, PublicKey},
    rpc, ExtendedSpendingKey, Namada, PaymentAddress,
};

use crate::{sdk::namada::Sdk, state::state::StateAddress};

pub const ADDRESS_PREFIX: &str = namada_sdk::string_encoding::ADDRESS_HRP;

#[derive(Clone, Debug, PartialEq)]
pub enum AccountIndentifier {
    Alias(String),
    Address(String),
    PublicKey(String),
    StateAddress(StateAddress),
    PaymentAddress(String),
    SpendingKey(String),
}

impl AccountIndentifier {
    pub async fn to_namada_address(&self, sdk: &Sdk) -> Address {
        match self {
            AccountIndentifier::Alias(alias) => {
                let wallet = sdk.namada.wallet.read().await;
                wallet.find_address(alias).unwrap().as_ref().clone()
            }
            AccountIndentifier::PublicKey(_) => {
                panic!()
            }
            AccountIndentifier::Address(address) => Address::decode(address).unwrap(),
            AccountIndentifier::StateAddress(metadata) => {
                Address::decode(metadata.address.clone()).unwrap()
            }
            AccountIndentifier::PaymentAddress(_) => unimplemented!(),
            AccountIndentifier::SpendingKey(_) => unimplemented!(),
        }
    }

    pub async fn to_payment_address(&self, sdk: &Sdk) -> PaymentAddress {
        match self {
            AccountIndentifier::Alias(alias) => {
                let wallet = sdk.namada.wallet.read().await;
                *wallet.find_payment_addr(alias).unwrap()
            }
            AccountIndentifier::Address(_) => unimplemented!(),
            AccountIndentifier::PublicKey(_) => unimplemented!(),
            AccountIndentifier::StateAddress(_) => unimplemented!(),
            AccountIndentifier::PaymentAddress(pa) => PaymentAddress::from_str(pa).unwrap(),
            AccountIndentifier::SpendingKey(_) => unimplemented!(),
        }
    }

    pub async fn to_spending_key(&self, sdk: &Sdk) -> ExtendedSpendingKey {
        match self {
            AccountIndentifier::Alias(alias) => {
                let mut wallet = sdk.namada.wallet.write().await;
                wallet.find_spending_key(alias, None).unwrap().key
            }
            AccountIndentifier::Address(_) => unimplemented!(),
            AccountIndentifier::PublicKey(_) => unimplemented!(),
            AccountIndentifier::StateAddress(_) => unimplemented!(),
            AccountIndentifier::PaymentAddress(_) => unimplemented!(),
            AccountIndentifier::SpendingKey(sk) => ExtendedSpendingKey::from_str(sk).unwrap(),
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
            AccountIndentifier::StateAddress(_) => unimplemented!(),
            AccountIndentifier::PaymentAddress(_) => unimplemented!(),
            AccountIndentifier::SpendingKey(_) => unimplemented!(),
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
                match address {
                    Address::Established(_) => panic!(),
                    Address::Implicit(_) => {
                        let wallet = sdk.namada.wallet.read().await;
                        let alias = wallet.find_alias(&address).unwrap();
                        alias.to_string()
                    }
                    Address::Internal(_) => panic!(),
                }
            }
            AccountIndentifier::PublicKey(public_key) => {
                return PublicKey::from_str(public_key).unwrap()
            }
            AccountIndentifier::StateAddress(_metadata) => unimplemented!(),
            AccountIndentifier::PaymentAddress(_) => unimplemented!(),
            AccountIndentifier::SpendingKey(_) => unimplemented!(),
        };
        let wallet = sdk.namada.wallet.read().await;
        wallet.find_public_key(&alias).unwrap()
    }

    pub async fn to_signing_keys(&self, sdk: &Sdk) -> Vec<common::PublicKey> {
        // We match alias first in order to avoid a wallet lock issue
        let alias = match self {
            AccountIndentifier::Alias(alias) => alias.clone(),
            AccountIndentifier::Address(address) => {
                let address = Address::decode(address).unwrap();
                match address {
                    Address::Established(_) => {
                        let account_info =
                            rpc::get_account_info(&sdk.namada.clone_client(), &address)
                                .await
                                .unwrap();
                        if let Some(account) = account_info {
                            return account.public_keys_map.pk_to_idx.keys().cloned().collect();
                        } else {
                            panic!()
                        }
                    }
                    Address::Implicit(_) => {
                        let wallet = sdk.namada.wallet.read().await;
                        let alias = wallet.find_alias(&address).unwrap();
                        alias.to_string()
                    }
                    Address::Internal(_) => panic!(),
                }
            }
            AccountIndentifier::PublicKey(public_key) => {
                return vec![PublicKey::from_str(public_key).unwrap()]
            }
            AccountIndentifier::StateAddress(_metadata) => unimplemented!(),
            AccountIndentifier::PaymentAddress(_) => unimplemented!(),
            AccountIndentifier::SpendingKey(_) => unimplemented!(),
        };
        let wallet = sdk.namada.wallet.read().await;
        vec![wallet.find_public_key(&alias).unwrap()]
    }
}
