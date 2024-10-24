use async_trait::async_trait;
use namada_sdk::{args::ClaimRewards, error::TxSubmitError, signing::default_sign, Namada};
use serde::{Deserialize, Serialize};

use super::{Task, TaskError, TaskParam};
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
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

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
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
        let tx = sdk
            .namada
            .submit(claim_reward_tx.clone(), &claim_rewards_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&claim_reward_tx, &tx) {
            match tx {
                Ok(tx) => {
                    let errors = Self::get_tx_errors(&claim_reward_tx, &tx).unwrap_or_default();
                    return Ok(StepResult::fail(errors));
                }
                Err(e) => match e {
                    namada_sdk::error::Error::Tx(TxSubmitError::AppliedTimeout) => {
                        return Err(TaskError::Timeout)
                    }
                    _ => return Ok(StepResult::fail(e.to_string())),
                },
            }
        }

        storage.add(
            TxClaimRewardsStorageKeys::ValidatorAddress.to_string(),
            validator.to_string(),
        );
        storage.add(
            TxClaimRewardsStorageKeys::DelegatorAddress.to_string(),
            delegator.to_string(),
        );

        Ok(StepResult::success(storage))
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
