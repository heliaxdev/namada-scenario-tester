use async_trait::async_trait;
use namada_sdk::{
    args::{SdkTypes, TxBuilder},
    rpc,
    tx::ProcessTxResponse,
    Namada,
};

use crate::{
    entity::address::AccountIndentifier,
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{
        settings::{TxSettings, TxSettingsDto},
        value::Value,
    },
};

pub mod become_validator;
pub mod bond;
pub mod change_metadata;
pub mod init_account;
pub mod init_default_proposal;
pub mod init_pgf_funding_proposal;
pub mod init_pgf_steward_proposal;
pub mod redelegate;
pub mod reveal_pk;
pub mod tx_transparent_transfer;
pub mod unbond;
pub mod vote;
pub mod wallet_new_key;
pub mod withdraw;

#[async_trait(?Send)]
pub trait Task {
    type P: TaskParam;
    type B: TxBuilder<SdkTypes>;

    async fn execute(
        &self,
        sdk: &Sdk,
        paramaters: Self::P,
        settings: TxSettings,
        state: &Storage,
    ) -> StepResult;

    async fn fetch_info(&self, sdk: &Sdk, step_storage: &mut StepStorage) {
        let block = rpc::query_block(sdk.namada.client())
            .await
            .unwrap()
            .unwrap();
        let epoch = rpc::query_epoch(sdk.namada.client()).await.unwrap();

        step_storage.add("epoch".to_string(), epoch.to_string());
        step_storage.add("height".to_string(), block.height.to_string());
    }

    async fn run(
        &self,
        sdk: &Sdk,
        dto: <<Self as Task>::P as TaskParam>::D,
        settings_dto: Option<TxSettingsDto>,
        state: &Storage,
    ) -> StepResult {
        let parameters = Self::P::parameter_from_dto(dto, state);
        let settings = Self::P::settings_from_dto(settings_dto, state);

        self.execute(sdk, parameters, settings, state).await
    }

    async fn add_settings(&self, sdk: &Sdk, builder: Self::B, settings: TxSettings) -> Self::B {
        if let Some(signers) = settings.signers {
            let mut signing_keys = vec![];
            for signer in signers {
                let public_key = signer.to_public_key(sdk).await;
                signing_keys.push(public_key)
            }
            builder.signing_keys(signing_keys)
        } else {
            builder
        }
    }

    fn get_tx_errors(tx_response: &ProcessTxResponse) -> Option<String> {
        match tx_response {
            ProcessTxResponse::Applied(result) => Some(result.log.clone()),
            _ => None,
        }
    }

    fn is_tx_rejected(tx_response: &Result<ProcessTxResponse, namada_sdk::error::Error>) -> bool {
        match tx_response {
            Ok(tx_result) => tx_result.is_applied_and_valid().is_none(),
            Err(_) => true,
        }
    }
}

pub trait TaskParam {
    type D;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self;
    fn settings_from_dto(dto: Option<TxSettingsDto>, _state: &Storage) -> TxSettings {
        let settings = if let Some(settings) = dto {
            settings
        } else {
            return TxSettings::default();
        };
        let broadcast_only = settings.broadcast_only.unwrap_or(false);
        let gas_token = match settings.gas_token.clone() {
            Some(Value::Value { value }) => Some(AccountIndentifier::Alias(value)),
            _ => None,
        };
        let gas_payer = match settings.gas_payer.clone() {
            Some(Value::Value { value }) => Some(AccountIndentifier::Alias(value)),
            _ => None,
        };
        let signers = settings.signers.clone().map(|signers| {
            signers
                .into_iter()
                .filter_map(|signer| match signer {
                    Value::Value { value } => Some(AccountIndentifier::Alias(value)),
                    _ => None,
                })
                .collect::<Vec<AccountIndentifier>>()
        });
        let expiration = match settings.expiration.clone() {
            Some(Value::Value { value }) => Some(value.parse::<u64>().unwrap()),
            _ => None,
        };
        let gas_limit = match settings.gas_limit.clone() {
            Some(Value::Value { value }) => Some(value.parse::<u64>().unwrap()),
            _ => None,
        };

        TxSettings {
            broadcast_only,
            gas_token,
            gas_payer,
            signers,
            expiration,
            gas_limit,
        }
    }
}
