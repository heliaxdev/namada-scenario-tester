use std::{path::PathBuf, str::FromStr};


use namada_sdk::{
    args::TxBuilder,
    core::types::{chain::ChainId, key::{common::SecretKey, PublicKeyHash}, address::{Address, ImplicitAddress}},
    io::{NullIo},
    masp::{fs::FsShieldedUtils, ShieldedContext},
    wallet::{fs::FsWalletUtils, Wallet, StoredKeypair},
    NamadaImpl,
};
use tendermint_rpc::{HttpClient};

use crate::config::AppConfig;

pub struct Sdk<'a> {
    pub base_dir: PathBuf,
    pub chain_id: String,
    pub rpc: String,
    pub namada: NamadaImpl<'a, HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
}

impl<'a> Sdk<'a> {
    pub async fn new(
        config: &'a AppConfig,
        base_dir: &'a PathBuf,
        http_client: &'a HttpClient,
        wallet: &'a mut Wallet<FsWalletUtils>,
        shielded_ctx: &'a mut ShieldedContext<FsShieldedUtils>,
        io: &'a NullIo,
    ) -> Sdk<'a> {
        let sk = SecretKey::from_str("00548aa8393422b88dce5f4be8ee0320638061c3e0649ada1b0dacbec4c0c75bb2").unwrap();
        let stored_keypair = StoredKeypair::Raw(sk.clone());
        let pk_hash = PublicKeyHash::from(&sk.to_public());
        let alias = "my_faucet".to_string();
        let public_key = sk.to_public();
        let address = Address::Implicit(ImplicitAddress::from(&public_key));
        wallet.insert_keypair(alias.clone(), stored_keypair, pk_hash, true);
        wallet.add_address(alias.clone(), address, true);
        // Setup the Namada context
        let namada = NamadaImpl::new(http_client, wallet, shielded_ctx, io)
            .await
            .expect("unable to construct Namada object")
            .chain_id(ChainId::from_str(&config.chain_id).unwrap());
        Self {
            base_dir: base_dir.to_owned(),
            chain_id: config.chain_id.to_owned(),
            rpc: config.rpc.to_owned(),
            namada
        }
    }
}
