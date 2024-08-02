use async_trait::async_trait;
use namada_sdk::{args::{ClaimRewards, TxBuilder}, signing::default_sign, Namada};
use serde::{Deserialize, Serialize};

use super::{Task, TaskParam};
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
    ) -> StepResult {
        let validator = parameters.source.to_namada_address(sdk).await;
        let delegator = parameters.delegator.to_namada_address(sdk).await;

        let mut claim_rewards_tx_builder = sdk.namada.new_claim_rewards(validator.clone());
        claim_rewards_tx_builder.source = Some(delegator.clone());
        claim_rewards_tx_builder = claim_rewards_tx_builder.force(true);

        let claim_rewards_tx_builder = self
            .add_settings(sdk, claim_rewards_tx_builder, settings)
            .await;

        let (mut claim_reward_tx, signing_data) = claim_rewards_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

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
            let errors = Self::get_tx_errors(&claim_reward_tx, &tx.unwrap()).unwrap_or_default();
            return StepResult::fail(errors);
        }

        storage.add(
            TxClaimRewardsStorageKeys::ValidatorAddress.to_string(),
            validator.to_string(),
        );
        storage.add(
            TxClaimRewardsStorageKeys::DelegatorAddress.to_string(),
            delegator.to_string(),
        );

        StepResult::success(storage)
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

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value, field } => {
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

        Self { source, delegator }
    }
}
