use async_trait::async_trait;
use namada_sdk::args::{TxBuilder, TxShieldingTransferData};
use namada_sdk::error::TxSubmitError;
use namada_sdk::signing::SigningTxData;
use namada_sdk::token::DenominatedAmount;
use namada_sdk::tx::data::GasLimit;
use namada_sdk::tx::Tx;
use namada_sdk::DEFAULT_GAS_LIMIT;
use namada_sdk::{
    args::{InputAmount, TxShieldingTransfer as NamadaTxShieldingTransfer},
    signing::default_sign,
    Namada,
};
use namada_sdk::{token, tx};
use serde::{Deserialize, Serialize};

use crate::utils::settings::TxSettings;
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskError, TaskParam};

pub enum TxShieldingTransferBatchStorageKeys {
    Source(usize),
    Target(usize),
    Amount(usize),
    Token(usize),
    BatchSize,
    AtomicBatch,
}

impl ToString for TxShieldingTransferBatchStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxShieldingTransferBatchStorageKeys::Source(idx) => format!("source-{}", idx),
            TxShieldingTransferBatchStorageKeys::Target(idx) => format!("target-{}", idx),
            TxShieldingTransferBatchStorageKeys::Amount(idx) => format!("amount-{}", idx),
            TxShieldingTransferBatchStorageKeys::Token(idx) => format!("token-{}", idx),
            TxShieldingTransferBatchStorageKeys::BatchSize => "batch-size".to_string(),
            TxShieldingTransferBatchStorageKeys::AtomicBatch => "batch-atomic".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxShieldingTransferBatch {}

impl TxShieldingTransferBatch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxShieldingTransferBatch {
    type P = TxShieldingTransferBatchParameters;
    type B = NamadaTxShieldingTransfer;

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
            let target_address = parameters.targets[param_idx].to_payment_address(sdk).await;
            let token_address = parameters.tokens[param_idx].to_namada_address(sdk).await;

            let token_amount = token::Amount::from_u64(parameters.amounts[param_idx]);
            let denominated_amount = DenominatedAmount::native(token_amount);

            let tx_transfer_data = TxShieldingTransferData {
                source: source_address.clone(),
                token: token_address.clone(),
                amount: InputAmount::Validated(denominated_amount),
            };

            let transfer_tx_builder =
                sdk.namada
                    .new_shielding_transfer(target_address, vec![tx_transfer_data]);

            let mut transfer_tx_builder =
                self.add_settings(sdk, transfer_tx_builder, settings.clone()).await;

            let res = transfer_tx_builder
                .build(&sdk.namada)
                .await
                .map_err(|e| TaskError::Build(e.to_string()));

            if res.is_ok() {
                storage.add(
                    TxShieldingTransferBatchStorageKeys::Source(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxShieldingTransferBatchStorageKeys::Target(param_idx).to_string(),
                    target_address.to_string(),
                );
                storage.add(
                    TxShieldingTransferBatchStorageKeys::Token(param_idx).to_string(),
                    token_address.to_string(),
                );
                storage.add(
                    TxShieldingTransferBatchStorageKeys::Amount(param_idx).to_string(),
                    token_amount.raw_amount().to_string(),
                );

                txs.push(res);
            }
        }

        let txs = txs
            .into_iter()
            .filter_map(|res| res.ok())
            .map(|(tx, signing_data, _masp_epoch)| {
                (tx, signing_data)
            })
            .collect::<Vec<(Tx, SigningTxData)>>();

        if txs.is_empty() {
            return Ok(StepResult::no_op());
        }

        let tx_args = Self::default_tx_arg(sdk).await;
        let gas_payer = settings.clone().gas_payer.unwrap().to_public_key(sdk).await;
        let tx_args = tx_args.gas_limit(GasLimit::from(
            settings.clone().gas_limit.unwrap_or(DEFAULT_GAS_LIMIT),
        ));
        let tx_args = tx_args.wrapper_fee_payer(gas_payer);
        let is_atomic = true;

        let (mut batch_tx, signing_datas) =
            tx::build_batch(txs.clone()).map_err(|e| TaskError::Build(e.to_string()))?;
        batch_tx.header.atomic = is_atomic;

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
            TxShieldingTransferBatchStorageKeys::BatchSize.to_string(),
            txs.len().to_string(),
        );
        storage.add(
            TxShieldingTransferBatchStorageKeys::AtomicBatch.to_string(),
            is_atomic.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxShieldingTransferBatchParametersDto {
    pub sources: Vec<Value>,
    pub targets: Vec<Value>,
    pub amounts: Vec<Value>,
    pub tokens: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxShieldingTransferBatchParameters {
    sources: Vec<AccountIndentifier>,
    targets: Vec<AccountIndentifier>,
    amounts: Vec<u64>,
    tokens: Vec<AccountIndentifier>,
}

impl TaskParam for TxShieldingTransferBatchParameters {
    type D = TxShieldingTransferBatchParametersDto;

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
