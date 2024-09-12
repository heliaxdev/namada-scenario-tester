use async_trait::async_trait;
use namada_sdk::{
    args::{InputAmount, TxBuilder, TxTransparentTransferData},
    error::TxSubmitError,
    signing::{default_sign, SigningTxData},
    token::{self, DenominatedAmount},
    tx::{self, data::GasLimit, Tx},
    Namada, DEFAULT_GAS_LIMIT,
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

pub enum TxTransparentTransferBatchStorageKeys {
    Source(usize),
    Target(usize),
    Amount(usize),
    Token(usize),
    BatchSize,
}

impl ToString for TxTransparentTransferBatchStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxTransparentTransferBatchStorageKeys::Source(entry) => format!("source-{}", entry),
            TxTransparentTransferBatchStorageKeys::Target(entry) => format!("target-{}", entry),
            TxTransparentTransferBatchStorageKeys::Amount(entry) => format!("amount-{}", entry),
            TxTransparentTransferBatchStorageKeys::Token(entry) => format!("amount-{}", entry),
            TxTransparentTransferBatchStorageKeys::BatchSize => "batch-size".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxTransparentTransferBatch {}

impl TxTransparentTransferBatch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxTransparentTransferBatch {
    type P = TxTransparentTransferBatchParameters;
    type B = namada_sdk::args::TxTransparentTransfer;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let mut storage = StepStorage::default();

        let batch_size = parameters.sources.len();

        let mut txs = vec![];

        for param_idx in 0..batch_size {
            let source_address = parameters.sources[param_idx].to_namada_address(sdk).await;
            let target_address = parameters.targets[param_idx].to_namada_address(sdk).await;
            let token_address = parameters.tokens[param_idx].to_namada_address(sdk).await;

            let token_amount = token::Amount::from_u64(parameters.amounts[param_idx]);

            let tx_transfer_data = TxTransparentTransferData {
                source: source_address.clone(),
                target: target_address.clone(),
                token: token_address.clone(),
                amount: InputAmount::Unvalidated(DenominatedAmount::native(token_amount)),
            };

            let transfer_tx_builder = sdk.namada.new_transparent_transfer(vec![tx_transfer_data]);

            let mut transfer_tx_builder = self
                .add_settings(sdk, transfer_tx_builder, settings.clone())
                .await;

            let res = transfer_tx_builder
                .build(&sdk.namada)
                .await
                .map_err(|e| TaskError::Build(e.to_string()));

            if res.is_ok() {
                storage.add(
                    TxTransparentTransferBatchStorageKeys::Source(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxTransparentTransferBatchStorageKeys::Target(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxTransparentTransferBatchStorageKeys::Token(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxTransparentTransferBatchStorageKeys::Amount(param_idx).to_string(),
                    token_amount.raw_amount().to_string(),
                );
            }

            txs.push(res);
        }

        let txs = txs
            .into_iter()
            .filter_map(|res| res.ok())
            .collect::<Vec<(Tx, SigningTxData)>>();

        let tx_args = Self::default_tx_arg(sdk).await;
        let gas_payer = settings.clone().gas_payer.unwrap().to_public_key(sdk).await;
        let tx_args = tx_args.gas_limit(GasLimit::from(
            settings.clone().gas_limit.unwrap_or(DEFAULT_GAS_LIMIT),
        ));
        let tx_args = tx_args.wrapper_fee_payer(gas_payer);

        let (mut batch_tx, signing_datas) =
            tx::build_batch(txs.clone()).map_err(|e| TaskError::Build(e.to_string()))?;

        for signing_data in signing_datas {
            sdk.namada
                .sign(&mut batch_tx, &tx_args, signing_data, default_sign, ())
                .await
                .expect("unable to sign tx");
        }

        let tx = sdk.namada.submit(batch_tx.clone(), &tx_args).await;

        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&batch_tx, &tx) {
            match tx {
                Ok(tx) => {
                    let errors = Self::get_tx_errors(&batch_tx, &tx).unwrap_or_default();
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
            TxTransparentTransferBatchStorageKeys::BatchSize.to_string(),
            txs.len().to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxTransparentTransferBatchParametersDto {
    pub sources: Vec<Value>,
    pub targets: Vec<Value>,
    pub tokens: Vec<Value>,
    pub amounts: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxTransparentTransferBatchParameters {
    pub sources: Vec<AccountIndentifier>,
    pub targets: Vec<AccountIndentifier>,
    pub tokens: Vec<AccountIndentifier>,
    pub amounts: Vec<u64>,
}

impl TaskParam for TxTransparentTransferBatchParameters {
    type D = TxTransparentTransferBatchParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let batch_size = dto.sources.len();
        let batch = (0..batch_size)
            .filter_map(|i| {
                let source = match dto.sources[i].clone() {
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
                let target = match dto.targets[i].clone() {
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
                let amount = match dto.amounts[i].clone() {
                    Value::Ref { value, field } => {
                        let was_step_successful = state.is_step_successful(&value);
                        if !was_step_successful {
                            return None;
                        }
                        state.get_step_item(&value, &field).parse::<u64>().unwrap()
                    }
                    Value::Value { value } => value.parse::<u64>().unwrap(),
                    Value::Fuzz { .. } => unimplemented!(),
                };
                let token = match dto.tokens[i].clone() {
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

                Some((source, target, token, amount))
            })
            .collect::<Vec<(
                AccountIndentifier,
                AccountIndentifier,
                AccountIndentifier,
                u64,
            )>>();

        Some(Self {
            sources: batch.iter().map(|t| t.0.clone()).collect(),
            targets: batch.iter().map(|t| t.1.clone()).collect(),
            tokens: batch.iter().map(|t| t.2.clone()).collect(),
            amounts: batch.iter().map(|t| t.3).collect(),
        })
    }
}
