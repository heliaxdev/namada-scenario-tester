use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::{args::ClaimRewards, signing::default_sign, Namada};
use serde::{Deserialize, Serialize};

use super::{BuildResult, Task, TaskError, TaskParam};
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

pub enum TxClaimRewardsStorageKeys {
    ValidatorAddress,
    DelegatorAddress,
}

impl ToString for TxClaimRewardsStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxClaimRewardsStorageKeys::ValidatorAddress => "validator-address".to_string(),
            TxClaimRewardsStorageKeys::DelegatorAddress => "delegator-address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxClaimRewards {}

impl TxClaimRewards {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxClaimRewards {
    type P = TxClaimRewardsteParameters;
    type B = ClaimRewards;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let validator = parameters.source.to_namada_address(sdk).await;
        let delegator = parameters.delegator.to_namada_address(sdk).await;

        let mut claim_rewards_tx_builder = sdk.namada.new_claim_rewards(validator.clone());
        claim_rewards_tx_builder.source = Some(delegator.clone());

        let claim_rewards_tx_builder = self
            .add_settings(sdk, claim_rewards_tx_builder, settings)
            .await;

        let (mut claim_reward_tx, signing_data) = claim_rewards_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut claim_reward_tx,
                &claim_rewards_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxClaimRewardsStorageKeys::ValidatorAddress.to_string(),
            validator.to_string(),
        );
        step_storage.add(
            TxClaimRewardsStorageKeys::DelegatorAddress.to_string(),
            delegator.to_string(),
        );

        Ok(BuildResult::new(
            claim_reward_tx,
            claim_rewards_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxClaimRewards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-claim-rewards")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxClaimRewardsteParametersDto {
    pub source: Value,
    pub delegator: Value,
}

#[derive(Clone, Debug)]
pub struct TxClaimRewardsteParameters {
    source: AccountIndentifier,
    delegator: AccountIndentifier,
}

impl TaskParam for TxClaimRewardsteParameters {
    type D = TxClaimRewardsteParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.source {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    "public-key" => AccountIndentifier::PublicKey(data),
                    "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                    _ => AccountIndentifier::Address(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };
        let delegator = match dto.delegator {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    "public-key" => AccountIndentifier::PublicKey(data),
                    "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                    _ => AccountIndentifier::Address(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };

        Some(Self { source, delegator })
    }
}
