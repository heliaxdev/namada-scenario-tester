use std::{str::FromStr};

use namada_sdk::{
    io::NullIo, masp::fs::FsShieldedUtils,
    wallet::fs::FsWalletUtils,
};
use tempfile::tempdir;
use tendermint_rpc::{HttpClient, Url};

use crate::{
    config::AppConfig,
    scenario::Scenario,
    sdk::namada::Sdk,
    state::state::Storage,
};

#[derive(Clone, Debug, Default)]
pub struct Runner {
    storage: Storage,
}

impl Runner {
    pub async fn run(&mut self, scenario: Scenario, config: &AppConfig) {
        let base_dir = tempdir().unwrap().path().to_path_buf();
        println!("Using directory: {}", base_dir.to_string_lossy());

        let url = Url::from_str(&config.rpc).expect("invalid RPC address");
        let http_client = HttpClient::new(url).unwrap();

        // Setup wallet storage
        let wallet_path = base_dir.join("wallet");
        let mut wallet = FsWalletUtils::new(wallet_path);

        // Setup shielded context storage
        let shielded_ctx_path = base_dir.join("/masp");
        let mut shielded_ctx = FsShieldedUtils::new(shielded_ctx_path);

        let io = NullIo;

        let sdk = Sdk::new(config, &base_dir, &http_client, &mut wallet, &mut shielded_ctx, &io).await;
        for _ in 0..config.runs {
            for step in &scenario.steps {
                let successful_prev_step = if step.id.eq(&0) {
                    true
                } else {
                    self.storage.is_step_successful(&(step.id - 1))
                };

                if successful_prev_step {
                    println!("Running step {}...", step.config);
                    let result = step.run(&self.storage, &sdk).await;
                    if result.is_succesful() {
                        println!("Step {} executed succesfully.", step.config);
                        self.storage.save_step_result(step.id, result)
                    } else {
                        println!("Step {} errored bepbop.", step.config);
                        self.storage.save_step_result(step.id, result)
                    }
                }
            }
        }
    }
}
