use async_trait::async_trait;
use namada_sdk::{
    args::TxBuilder,
    error::TxSubmitError,
    signing::{default_sign, SigningTxData},
    token,
    tx::{self, data::GasLimit, Tx},
    Namada, DEFAULT_GAS_LIMIT,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    queries::validators::ValidatorsQueryStorageKeys,
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

pub enum TxBondBatchStorageKeys {
    SourceAddress(usize),
    ValidatorAddress(usize),
    Amount(usize),
    BatchSize,
    AtomicBatch,
}

impl ToString for TxBondBatchStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxBondBatchStorageKeys::SourceAddress(entry) => format!("source-{}-address", entry),
            TxBondBatchStorageKeys::ValidatorAddress(entry) => format!("validator-{}-address", entry),
            TxBondBatchStorageKeys::Amount(entry) => format!("amount-{}", entry),
            TxBondBatchStorageKeys::BatchSize => "batch-size".to_string(),
            TxBondBatchStorageKeys::AtomicBatch => "batch-atomic".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxBondBatch {}

impl TxBondBatch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxBondBatch {
    type P = TxBondBatchParameters;
    type B = namada_sdk::args::Bond;

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

            let token_amount = token::Amount::from_u64(parameters.amounts[param_idx]);

            let transfer_tx_builder = sdk
                .namada
                .new_bond(target_address.clone(), token_amount)
                .source(source_address.clone());

            let transfer_tx_builder = self
                .add_settings(sdk, transfer_tx_builder, settings.clone())
                .await;

            let res = transfer_tx_builder
                .build(&sdk.namada)
                .await
                .map_err(|e| TaskError::Build(e.to_string()));

            if res.is_ok() {
                storage.add(
                    TxBondBatchStorageKeys::SourceAddress(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxBondBatchStorageKeys::ValidatorAddress(param_idx).to_string(),
                    target_address.to_string(),
                );
                storage.add(
                    TxBondBatchStorageKeys::Amount(param_idx).to_string(),
                    token_amount.raw_amount().to_string(),
                );
                txs.push(res);
            }
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
            TxBondBatchStorageKeys::BatchSize.to_string(),
            txs.len().to_string(),
        );
        storage.add(
            TxBondBatchStorageKeys::AtomicBatch.to_string(),
            is_atomic.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxBondBatchParametersDto {
    pub sources: Vec<Value>,
    pub targets: Vec<Value>,
    pub amounts: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxBondBatchParameters {
    pub sources: Vec<AccountIndentifier>,
    pub targets: Vec<AccountIndentifier>,
    pub amounts: Vec<u64>,
}

impl TaskParam for TxBondBatchParameters {
    type D = TxBondBatchParametersDto;

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
                    Value::Fuzz { value } => {
                        let step_id = value.expect("Bond task requires fuzz for source to define the step id to a validator query step");
                        let total_validators = state
                            .get_step_item(
                                &step_id,
                                ValidatorsQueryStorageKeys::TotalValidator
                                    .to_string()
                                    .as_str(),
                            )
                            .parse::<u64>()
                            .unwrap();

                        let validator_idx = rand::thread_rng().gen_range(0..total_validators);

                        let validator_address = state.get_step_item(
                            &step_id,
                            ValidatorsQueryStorageKeys::Validator(validator_idx)
                                .to_string()
                                .as_str(),
                        );

                        AccountIndentifier::Address(validator_address)
                    }
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
                    Value::Fuzz { value } => {
                        let step_id = value.expect("Bond task requires fuzz for source to define the step id to a validator query step");
                        let total_validators = state
                            .get_step_item(
                                &step_id,
                                ValidatorsQueryStorageKeys::TotalValidator
                                    .to_string()
                                    .as_str(),
                            )
                            .parse::<u64>()
                            .unwrap();

                        let validator_idx = rand::thread_rng().gen_range(0..total_validators);

                        let validator_address = state.get_step_item(
                            &step_id,
                            ValidatorsQueryStorageKeys::Validator(validator_idx)
                                .to_string()
                                .as_str(),
                        );

                        AccountIndentifier::Address(validator_address)
                    }
                };
                let amount = match dto.amounts[i].clone() {
                    Value::Ref { value, field } => {
                        let was_step_successful = state.is_step_successful(&value);
                        if !was_step_successful {
                            return None;
                        }
                        let amount = state.get_step_item(&value, &field);
                        amount.parse::<u64>().unwrap()
                    }
                    Value::Value { value } => value.parse::<u64>().unwrap(),
                    Value::Fuzz { .. } => unimplemented!(),
                };

                Some((source, target, amount))
            })
            .collect::<Vec<(
                AccountIndentifier,
                AccountIndentifier,
                u64,
            )>>();

        Some(Self {
            sources: batch.iter().map(|t| t.0.clone()).collect(),
            targets: batch.iter().map(|t| t.1.clone()).collect(),
            amounts: batch.iter().map(|t| t.2).collect(),
        })
    }
}
