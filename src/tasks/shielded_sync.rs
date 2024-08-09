use std::str::FromStr;

use async_trait::async_trait;
use namada_sdk::args::Bond;
use namada_sdk::control_flow::install_shutdown_signal;
use namada_sdk::io::DevNullProgressBar;
use namada_sdk::masp::utils::{IndexerMaspClient, LedgerMaspClient};
use namada_sdk::masp::ShieldedSyncConfig;
use namada_sdk::masp::{find_valid_diversifier, MaspLocalTaskEnv, PaymentAddress};
use namada_sdk::masp_primitives::zip32;
use namada_sdk::masp_primitives::zip32::ExtendedFullViewingKey;
use namada_sdk::Namada;
use namada_sdk::{address::Address, key::SchemeType};
use rand::rngs::OsRng;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::{Task, TaskError, TaskParam};
use crate::utils::settings::TxSettings;
use crate::utils::value::Value;
use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
};
use namada_sdk::key::RefTo;

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
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let vks: Vec<_> = sdk
            .namada
            .wallet()
            .await
            .get_viewing_keys()
            .values()
            .map(|evk| ExtendedFullViewingKey::from(*evk).fvk.vk)
            .collect();

        let mut shielded_ctx = sdk.namada.shielded_mut().await;

        let masp_client = LedgerMaspClient::new(sdk.namada.clone_client());
        // let masp_client = IndexerMaspClient::new(
        //     reqwest::Client::new(),
        //     Url::from_str("https://masp.public.heliax.work/internal-devnet-it.14814a0e13c")
        //         .unwrap(),
        //     true,
        // );
        let task_env =
            MaspLocalTaskEnv::new(4).map_err(|e| TaskError::ShieldedSync(e.to_string()))?;
        let shutdown_signal = install_shutdown_signal();
        let config = ShieldedSyncConfig::builder()
            .client(masp_client)
            .fetched_tracker(DevNullProgressBar)
            .scanned_tracker(DevNullProgressBar)
            .build();

        shielded_ctx
            .fetch(shutdown_signal, task_env, config, None, &[], &vks)
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
