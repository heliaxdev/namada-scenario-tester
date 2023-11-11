use std::{path::PathBuf, str::FromStr};

use namada_sdk::{
    args::TxBuilder,
    core::types::{
        address::{Address, ImplicitAddress},
        chain::ChainId,
        key::{common::SecretKey, PublicKeyHash},
    },
    io::NullIo,
    masp::{fs::FsShieldedUtils, ShieldedContext},
    wallet::{fs::FsWalletUtils, StoredKeypair, Wallet},
    NamadaImpl,
};
use tendermint_rpc::HttpClient;

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
        // Insert the faucet keypair into the wallet
        let sk = SecretKey::from_str(&config.faucet_sk).unwrap();
        let stored_keypair = StoredKeypair::Raw(sk.clone());
        let pk_hash = PublicKeyHash::from(&sk.to_public());
        let alias = "faucet".to_string();
        let public_key = sk.to_public();
        let address = Address::Implicit(ImplicitAddress::from(&public_key));
        wallet.insert_keypair(alias.clone(), stored_keypair, pk_hash, true);
        wallet.add_address(alias.clone(), address, true);

        let namada = NamadaImpl::new(http_client, wallet, shielded_ctx, io)
            .await
            .expect("unable to construct Namada object")
            .chain_id(ChainId::from_str(&config.chain_id).unwrap());

        Self {
            base_dir: base_dir.to_owned(),
            chain_id: config.chain_id.to_owned(),
            rpc: config.rpc.to_owned(),
            namada,
        }
    }

    pub async fn find_secret_key(&self, alias: &str) -> SecretKey {
        let mut wallet = self.namada.wallet.write().await;
        wallet.find_key(alias, None).unwrap()
    }
}
