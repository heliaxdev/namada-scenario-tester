use async_trait::async_trait;
use namada_sdk::args::Bond;
use namada_sdk::masp::utils::{DefaultTracker, LedgerMaspClient, RetryStrategy};
use namada_sdk::masp::{find_valid_diversifier, PaymentAddress};
use namada_sdk::masp_primitives::zip32;
use namada_sdk::masp_primitives::zip32::ExtendedFullViewingKey;
use namada_sdk::Namada;
use namada_sdk::{address::Address, key::SchemeType};
use rand::rngs::OsRng;
use rand::{distributions::Alphanumeric, Rng};
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

        shielded_ctx
            .fetch(
                LedgerMaspClient::new(sdk.namada.client()),
                &DefaultTracker::new(sdk.namada.io()),
                None,
                None,
                RetryStrategy::Forever,
                &[],
                &vks,
            )
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
