use std::{path::PathBuf, str::FromStr};

use namada_sdk::{
    args::TxBuilder,
    core::types::{
        address::{Address, ImplicitAddress},
        chain::ChainId,
        key::{common::{SecretKey, PublicKey}},
    },
    io::NullIo,
    masp::{fs::FsShieldedUtils, ShieldedContext},
    wallet::{fs::FsWalletUtils, Wallet},
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
        let alias = "faucet".to_string();
        let public_key = sk.to_public();
        let address = Address::Implicit(ImplicitAddress::from(&public_key));
        wallet.insert_keypair(alias.clone(), true, sk.clone(), None, Some(address), None);

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
        wallet.find_secret_key(alias, None).unwrap()
    }

    pub async fn find_public_key(&self, alias_or_pkh: impl AsRef<str>) -> PublicKey {
        let wallet = self.namada.wallet.write().await;
        wallet.find_public_key(alias_or_pkh).unwrap()
    }
}
