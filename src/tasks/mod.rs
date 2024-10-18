use std::{path::PathBuf, str::FromStr};

use async_trait::async_trait;
use namada_sdk::{
    args::{self, DeviceTransport, SdkTypes, TxBuilder},
    rpc::{self},
    state::Epoch,
    tx::{data::GasLimit, either, ProcessTxResponse, Tx, TX_REVEAL_PK},
    Namada, DEFAULT_GAS_LIMIT,
};
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

pub mod become_validator;
pub mod bond;
pub mod bond_batch;
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
pub mod transparent_transfer_batch;
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
}

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
    ) -> Result<StepResult, TaskError>;

    async fn fetch_info(&self, sdk: &Sdk, step_storage: &mut StepStorage) {
        let block_height = match rpc::query_block(&sdk.namada.clone_client()).await {
            Ok(Some(block)) => block.height.to_string(),
            Err(e) => {
                println!("{}", e);
                0.to_string()
            }
            _ => 0.to_string(),
        };

        let epoch = match rpc::query_epoch(&sdk.namada.clone_client()).await {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                Epoch(0)
            }
        };

        step_storage.add("epoch".to_string(), epoch.to_string());
        step_storage.add("height".to_string(), block_height.to_string());
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

        match self.execute(sdk, parameters, settings, state).await {
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
        let builder = builder.tx(|x| args::Tx {
            memo: Some(
                "Tx sent from scenario tester"
                    .to_string()
                    .as_bytes()
                    .to_vec(),
            ),
            ..x
        });
        let builder = builder.gas_limit(GasLimit::from(settings.gas_limit.unwrap_or(300000)));

        if let Some(account) = settings.gas_payer {
            let public_key = account.to_public_key(sdk).await;
            builder.wrapper_fee_payer(public_key)
        } else {
            builder
        }
    }

    // we could do this better by returning all the errors an not just the first one we see
    fn get_tx_errors(tx: &Tx, tx_response: &ProcessTxResponse) -> Option<String> {
        let wrapper_hash = tx.wrapper_hash();
        for cmt in tx.header.batch.clone() {
            match tx_response {
                ProcessTxResponse::Applied(result) => match &result.batch {
                    Some(batch) => {
                        println!("batch result: {:#?}", batch);
                        match batch.get_inner_tx_result(wrapper_hash.as_ref(), either::Right(&cmt))
                        {
                            Some(Ok(res)) => {
                                let errors = res.vps_result.errors.clone();
                                let _status_flag = res.vps_result.status_flags;
                                let _rejected_vps = res.vps_result.rejected_vps.clone();
                                return Some(serde_json::to_string(&errors).unwrap());
                            }
                            Some(Err(e)) => return Some(e.to_string()),
                            _ => (),
                        }
                    }
                    None => (),
                },
                _ => (),
            }
        }
        None
    }

    fn is_tx_rejected(
        tx: &Tx,
        tx_response: &Result<ProcessTxResponse, namada_sdk::error::Error>,
    ) -> bool {
        let wrapper_hash = tx.wrapper_hash();
        for commitment in tx.header.batch.clone() {
            let is_invalid = match tx_response {
                Ok(tx_result) => tx_result
                    .is_applied_and_valid(wrapper_hash.as_ref(), &commitment)
                    .is_none(),
                Err(_) => true,
            };
            if is_invalid {
                return true;
            }
        }
        false
    }

    async fn default_tx_arg(sdk: &Sdk) -> args::Tx {
        let wallet = sdk.namada.wallet.read().await;
        let nam = wallet
            .find_address("nam")
            .expect("Native token should be present.")
            .into_owned();

        args::Tx {
            dry_run: false,
            dry_run_wrapper: false,
            dump_tx: false,
            output_folder: None,
            force: false,
            broadcast_only: false,
            ledger_address: tendermint_rpc::Url::from_str("http://127.0.0.1:26657").unwrap(),
            initialized_account_alias: None,
            wallet_alias_force: false,
            fee_amount: None,
            wrapper_fee_payer: None,
            fee_token: nam,
            gas_limit: GasLimit::from(DEFAULT_GAS_LIMIT),
            expiration: Default::default(),
            chain_id: None,
            signing_keys: vec![],
            signatures: vec![],
            tx_reveal_code_path: PathBuf::from(TX_REVEAL_PK),
            password: None,
            memo: None,
            use_device: false,
            device_transport: DeviceTransport::default(),
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
