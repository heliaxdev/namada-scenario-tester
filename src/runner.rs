use std::{collections::HashMap, str::FromStr, thread, time::Duration};

use namada_sdk::{
    io::NullIo, masp::fs::FsShieldedUtils, queries::Client, wallet::fs::FsWalletUtils,
};
use tempfile::tempdir;
use tendermint_rpc::{HttpClient, Url};

use crate::{
    antithesis::AntithesisReport, config::AppConfig, report::Report, scenario::Scenario, sdk::namada::Sdk, state::state::Storage
};

#[derive(Clone, Debug, Default)]
pub struct Runner {
    storage: Storage,
}

impl Runner {
    pub async fn run(
        &mut self,
        worker_id: u64,
        scenario: Scenario,
        config: &AppConfig,
        scenario_name: String,
    ) {
        let base_dir = tempdir().unwrap().path().to_path_buf();
        println!("Using directory: {}", base_dir.to_string_lossy());
        println!("Using scenario: {}", scenario_name);

        let url = Url::from_str(&config.rpc).expect("invalid RPC address");
        let http_client = HttpClient::new(url).unwrap();

        // Setup antithesis report struct
        let antithesis_report = AntithesisReport::default();

        // Setup wallet storage
        let wallet_path = base_dir.join("wallet");
        let wallet = FsWalletUtils::new(wallet_path);

        // Setup shielded context storage
        let shielded_ctx_path = base_dir.join("masp");
        let shielded_ctx = FsShieldedUtils::new(shielded_ctx_path);

        let io = NullIo;

        let sdk = Sdk::new(
            config,
            &base_dir,
            http_client.clone(),
            wallet,
            shielded_ctx,
            io,
        )
        .await;
        let scenario_settings = &scenario.settings;

        // Wait for the first 2 blocks
        loop {
            let latest_blocked = http_client.latest_block().await;
            if let Ok(block) = latest_blocked {
                if block.block.header.height.value() > 2 {
                    break;
                }
            } else {
                thread::sleep(Duration::from_secs(10));
            }
        }

        for try_index in 0..=scenario_settings.retry_for.unwrap_or_default() {
            for step in &scenario.steps {
                println!(
                    "Worker id {} running step {} ({})...",
                    worker_id, step.config, step.id
                );
                let result = step.run(&self.storage, &sdk).await;
                if result.is_succesful() {
                    println!(
                        "Worker id {} step {} executed succesfully.",
                        worker_id, step.config
                    );
                    self.storage.save_step_result(step.id, result)
                } else if result.is_fail() {
                    println!(
                        "Worker id {} step {} errored bepbop: error is <{}>.",
                        worker_id,
                        step.config,
                        result.fail_error()
                    );
                    self.storage.save_step_result(step.id, result)
                } else {
                    println!(
                        "Worker id {} step check {} errored riprip: {}",
                        worker_id, step.config, result.outcome
                    );
                    self.storage.save_step_result(step.id, result);
                    break;
                }
                println!();
            }

            if let (
                Some(report_url),
                Some(sha),
                Some(minio_url),
                Some(minio_access_key),
                Some(minio_secret_key),
                Some(artifacts_url),
            ) = (
                &config.report_url,
                &config.sha,
                &config.minio_url,
                &config.minio_access_key,
                &config.minio_secret_key,
                &config.artifacts_url,
            ) {
                let mut sha_short = sha.clone();
                sha_short.truncate(8);
                let scenario_name = &scenario_name.replace(".json", "").replace("scenarios/", "");
                let report_name = format!(
                    "report-{}-{}-{}-{}.md",
                    config.chain_id, scenario_name, sha_short, try_index
                );

                println!("Building report...");

                let (report_path, outcome) = Report::new(
                    config,
                    self.storage.clone(),
                    scenario.clone(),
                )
                .generate_report(&base_dir, &report_name, scenario_name);

                println!("Uploading report...");

                Report::upload_report(
                    minio_url,
                    minio_access_key,
                    minio_secret_key,
                    &report_name,
                    &report_path,
                )
                .await;

                Report::update_commit_status(
                    report_url,
                    artifacts_url,
                    &outcome,
                    sha,
                    &report_name,
                    scenario_name,
                )
                .await;
            } else {
                println!("Skipping report submission.");
            }
        }

        println!("Done.");
    }
}
