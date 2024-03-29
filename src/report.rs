use std::{
    fs::File,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use markdown_gen::markdown::{AsMarkdown, List, Markdown};
use minio::s3::{args::UploadObjectArgs, client::Client, creds::StaticProvider, http::BaseUrl};
use serde::Serialize;

use crate::{config::AppConfig, scenario::Scenario, state::state::Storage};

#[derive(Clone, Debug, Serialize)]
pub struct StatusBody {
    #[serde(rename(serialize = "commit_sha"))]
    pub sha: String,
    #[serde(rename(serialize = "repo_owner"))]
    pub owner: String,
    pub repo: String,
    pub state: String,
    pub description: String,
    pub context: String,
}

pub struct Report {
    pub config: AppConfig,
    pub storage: Storage,
    pub scenario: Scenario,
}

impl Report {
    pub fn new(config: &AppConfig, storage: Storage, scenario: Scenario) -> Self {
        Self {
            config: config.clone(),
            storage,
            scenario,
        }
    }
    pub fn generate_report(
        &self,
        base_dir: &Path,
        name: &str,
        scenario_name: &str,
    ) -> (PathBuf, String) {
        let report_path = base_dir.join(name);

        let file_report = File::create(&report_path).unwrap();
        let mut md = Markdown::new(file_report);

        md.write("Info".heading(2)).unwrap();

        let outcome = self.storage.is_succesful().to_string();
        let info_list = List::new(false)
            .item("Chain ID: ".paragraph().append(self.config.chain_id.code()))
            .item("RPC url: ".paragraph().append(self.config.rpc.code()))
            .item("Outcome: ".paragraph().append(outcome.code()))
            .item("Scenario: ".paragraph().append(scenario_name.code()))
            .item(
                "Software version: "
                    .paragraph()
                    .append(env!("VERGEN_GIT_SHA").code()),
            );

        md.write(info_list).unwrap();
        md.write("\n").unwrap();

        md.write("Steps".heading(2)).unwrap();
        for (id, step_outcome) in self
            .storage
            .step_results
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let step = self.scenario.steps.get(*id as usize).unwrap();
            let step_type = step.config.to_string();

            let outcome = step_outcome.to_string();
            let step_id = id.to_string();

            let step_list = List::new(false)
                .title("Step id: ".paragraph().append(step_id.code()))
                .item("Type: ".paragraph().append(step_type.code()))
                .item("Outcome: ".paragraph().append(outcome.code()));

            md.write(step_list).unwrap();
            md.write("\n").unwrap();
        }

        (report_path, outcome)
    }

    pub async fn upload_report(
        minio_url: &str,
        minio_access_key: &str,
        minio_secret_key: &str,
        name: &str,
        report_path: &Path,
    ) {
        let base_url = minio_url.parse::<BaseUrl>().unwrap();

        let static_provider = StaticProvider::new(minio_access_key, minio_secret_key, None);

        let client = Client::new(
            base_url.clone(),
            Some(Box::new(static_provider)),
            None,
            None,
        )
        .unwrap();

        client
            .upload_object(
                &UploadObjectArgs::new(
                    "scenario-testing-results",
                    name,
                    report_path.to_str().unwrap(),
                )
                .unwrap(),
            )
            .await
            .unwrap();
    }

    pub async fn update_commit_status(
        report_url: &str,
        artifacts_url: &str,
        state: &str,
        sha: &str,
        report_name: &str,
        scenario_name: &str,
    ) {
        let client = reqwest::Client::new();

        let status_url = format!("{}/v1/report/status", report_url);
        let status_body = StatusBody {
            sha: sha.to_string(),
            owner: "anoma".to_string(),
            repo: "namada".to_string(),
            state: state.to_string(),
            description: format!("{}/scenario-testing-results/{}", artifacts_url, report_name),
            context: format!("Scenario {}", scenario_name),
        };

        client
            .post(&status_url)
            .json(&status_body)
            .send()
            .await
            .unwrap();
    }
}
