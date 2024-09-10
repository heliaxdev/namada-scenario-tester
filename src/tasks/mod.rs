use async_trait::async_trait;
use namada_sdk::{
    args::{self, SdkTypes, TxBuilder},
    error::TxSubmitError,
    rpc::{self, TxResponse},
    state::Epoch,
    tx::{data::GasLimit, either, ProcessTxResponse, Tx},
    Namada,
};
use std::fmt::Display;
use thiserror::Error;

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

pub mod batch;
pub mod become_validator;
pub mod bond;
pub mod change_consensus_key;
pub mod change_metadata;
pub mod claim_rewards;
pub mod deactivate_validator;
pub mod init_account;
pub mod init_default_proposal;
pub mod init_pgf_funding_proposal;
pub mod init_pgf_steward_proposal;
pub mod reactivate_validator;
pub mod redelegate;
pub mod reveal_pk;
pub mod shielded_sync;
pub mod tx_shielded_transfer;
pub mod tx_shielding_transfer;
pub mod tx_transparent_transfer;
pub mod tx_unshielding_transfer;
pub mod unbond;
pub mod update_account;
pub mod vote;
pub mod wallet_new_key;
pub mod withdraw;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("error waiting for timeout")]
    Timeout,
    #[error("error building tx `{0}`")]
    Build(String),
    #[error("error fetching shielded context data `{0}`")]
    ShieldedSync(String),
    #[error("error saving wallet")]
    Wallet,
}

pub struct BuildResult {
    pub tx_data: Option<(Tx, args::Tx)>,
    pub step_storage: StepStorage,
}

impl BuildResult {
    pub fn new(tx: Tx, tx_args: args::Tx, step_storage: StepStorage) -> Self {
        Self {
            tx_data: Some((tx, tx_args)),
            step_storage,
        }
    }

    pub fn empty(step_storage: StepStorage) -> Self {
        Self {
            tx_data: None,
            step_storage,
        }
    }

    pub fn should_execute(&self) -> bool {
        self.tx_data.is_some()
    }

    pub fn execute_data(&self) -> (Tx, args::Tx, StepStorage) {
        if let Some((tx, tx_args)) = self.tx_data.clone() {
            (tx, tx_args, self.step_storage.clone())
        } else {
            panic!()
        }
    }
}

#[async_trait(?Send)]
pub trait Task: Display {
    type P: TaskParam;
    type B: TxBuilder<SdkTypes>;

    async fn execute(
        &self,
        sdk: &Sdk,
        data: BuildResult,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let (tx, tx_args, mut step_storage) = if data.should_execute() {
            data.execute_data()
        } else {
            return Ok(StepResult::success(data.step_storage));
        };

        let tx_result = sdk.namada.submit(tx.clone(), &tx_args).await;

        let Ok(ProcessTxResponse::Applied(TxResponse { height, .. })) = &tx_result else {
            unreachable!()
        };

        if Self::is_tx_rejected(&tx, &tx_result) {
            match tx_result {
                Ok(tx_result) => {
                    let errors = Self::get_tx_errors(&tx, &tx_result).unwrap_or_default();
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

        step_storage.add("height".to_string(), height.to_string());
        step_storage.add("step-type".to_string(), self.to_string());

        Ok(StepResult::success(step_storage))
    }

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError>;

    async fn fetch_info(&self, sdk: &Sdk, step_storage: &mut StepStorage) -> Epoch {
        let epoch = match rpc::query_epoch(sdk.namada.client()).await {
            Ok(res) => res,
            Err(_e) => Epoch(0),
        };

        step_storage.add("epoch".to_string(), epoch.to_string());

        epoch
    }

    async fn run(
        &self,
        sdk: &Sdk,
        dto: <<Self as Task>::P as TaskParam>::D,
        settings_dto: Option<TxSettingsDto>,
        state: &Storage,
    ) -> StepResult {
        let parameters = if let Some(parameters) = Self::P::parameter_from_dto(dto, state) {
            parameters
        } else {
            return StepResult::no_op();
        };
        let settings = Self::P::settings_from_dto(settings_dto, state);

        let build_data = match self.build(sdk, parameters, settings).await {
            Ok(data) => data,
            Err(e) => {
                match e {
                    TaskError::Build(e) => {
                        println!("tx build error: {e}");
                    }
                    TaskError::ShieldedSync(e) => {
                        println!("shielded sync error: {e}");
                    }
                    TaskError::Timeout => {
                        println!("timeout waiting for tx to be applied");
                    }
                    TaskError::Wallet => {
                        println!("Error saving wallet")
                    }
                }
                return StepResult::no_op();
            }
        };

        match self.execute(sdk, build_data, state).await {
            Ok(step_result) => step_result,
            Err(e) => {
                match e {
                    TaskError::Build(e) => {
                        println!("tx build error: {e}");
                    }
                    TaskError::ShieldedSync(e) => {
                        println!("shielded sync error: {e}");
                    }
                    TaskError::Timeout => {
                        println!("timeout waiting for tx to be applied");
                    }
                    TaskError::Wallet => {
                        println!("Error saving wallet")
                    }
                }
                StepResult::no_op()
            }
        }
    }

    async fn add_settings(&self, sdk: &Sdk, builder: Self::B, settings: TxSettings) -> Self::B {
        let builder = if let Some(signers) = settings.signers {
            if signers.is_empty() {
                return builder;
            }
            let mut signing_keys = vec![];
            for signer in signers {
                let public_key = signer.to_public_key(sdk).await;
                signing_keys.push(public_key)
            }
            let builder = builder.signing_keys(signing_keys.clone());
            builder.wrapper_fee_payer(signing_keys.first().unwrap().clone())
        } else {
            builder
        };
        let builder = builder.gas_limit(GasLimit::from(settings.gas_limit.unwrap_or(300000)));

        if let Some(account) = settings.gas_payer {
            let public_key = account.to_public_key(sdk).await;
            builder.wrapper_fee_payer(public_key)
        } else {
            builder
        }
    }

    fn get_tx_errors(tx: &Tx, tx_response: &ProcessTxResponse) -> Option<String> {
        let cmt = tx.first_commitments().unwrap().to_owned();
        let wrapper_hash = tx.wrapper_hash();
        match tx_response {
            ProcessTxResponse::Applied(result) => match &result.batch {
                Some(batch) => {
                    println!("batch result: {:#?}", batch);
                    match batch.get_inner_tx_result(wrapper_hash.as_ref(), either::Right(&cmt)) {
                        Some(Ok(res)) => {
                            let errors = res.vps_result.errors.clone();
                            let _status_flag = res.vps_result.status_flags;
                            let _rejected_vps = res.vps_result.rejected_vps.clone();
                            Some(serde_json::to_string(&errors).unwrap())
                        }
                        Some(Err(e)) => Some(e.to_string()),
                        _ => None,
                    }
                }
                None => None,
            },
            _ => None,
        }
    }

    fn is_tx_rejected(
        tx: &Tx,
        tx_response: &Result<ProcessTxResponse, namada_sdk::error::Error>,
    ) -> bool {
        let cmt = tx.first_commitments().unwrap().to_owned();
        let wrapper_hash = tx.wrapper_hash();
        match tx_response {
            Ok(tx_result) => tx_result
                .is_applied_and_valid(wrapper_hash.as_ref(), &cmt)
                .is_none(),
            Err(_) => true,
        }
    }
}

pub trait TaskParam: Sized {
    type D;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self>;
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
