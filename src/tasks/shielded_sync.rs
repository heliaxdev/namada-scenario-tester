use async_trait::async_trait;
use namada_sdk::args::Bond;
use namada_sdk::control_flow::install_shutdown_signal;
use namada_sdk::io::DevNullProgressBar;
use namada_sdk::masp::utils::LedgerMaspClient;
use namada_sdk::masp::MaspLocalTaskEnv;
use namada_sdk::masp::ShieldedSyncConfig;
use namada_sdk::Namada;
use serde::{Deserialize, Serialize};

use super::{Task, TaskError, TaskParam};
use crate::utils::settings::TxSettings;
use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage};

#[derive(Clone, Debug, Default)]
pub struct ShieldedSync {}

impl ShieldedSync {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for ShieldedSync {
    type P = ShieldedSyncParameters;
    type B = Bond; // just a placeholder

    async fn execute(
        &self,
        sdk: &Sdk,
        _dto: Self::P,
        _settings: TxSettings,
        state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let maybe_height_to_sync = state.get_last_masp_tx_height();

        let vks: Vec<_> = sdk
            .namada
            .wallet()
            .await
            .get_viewing_keys()
            .values()
            .map(|evk| evk.map(|key| key.as_viewing_key()))
            .collect();

        let mut shielded_ctx = sdk.namada.shielded_mut().await;

        let masp_client = LedgerMaspClient::new(sdk.namada.clone_client(), 100);
        let task_env =
            MaspLocalTaskEnv::new(4).map_err(|e| TaskError::ShieldedSync(e.to_string()))?;
        let shutdown_signal = install_shutdown_signal(true);

        let config = ShieldedSyncConfig::builder()
            .client(masp_client)
            .fetched_tracker(DevNullProgressBar)
            .scanned_tracker(DevNullProgressBar)
            .applied_tracker(DevNullProgressBar)
            .shutdown_signal(shutdown_signal);

        let config = if maybe_height_to_sync.is_some() {
            config.wait_for_last_query_height(true).build()
        } else {
            config.build()
        };

        shielded_ctx
            .sync(task_env, config, maybe_height_to_sync, &[], &vks)
            .await
            .map_err(|e| TaskError::ShieldedSync(e.to_string()))?;

        Ok(StepResult::default())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShieldedSyncParametersDto;

#[derive(Clone, Debug)]
pub struct ShieldedSyncParameters;

impl TaskParam for ShieldedSyncParameters {
    type D = ShieldedSyncParametersDto;

    fn parameter_from_dto(_dto: Self::D, _state: &Storage) -> Option<Self> {
        Some(ShieldedSyncParameters)
    }
}
