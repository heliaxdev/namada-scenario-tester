use async_trait::async_trait;
use namada_sdk::{
    args::TxBuilder,
    error::TxSubmitError,
    signing::{default_sign, SigningTxData},
    token::Amount,
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

pub enum TxRedelegateBatchStorageKeys {
    SourceValidatorAddress(usize),
    DestValidatorAddress(usize),
    SourceAddress(usize),
    Amount(usize),
    BatchSize,
    AtomicBatch,
}

impl ToString for TxRedelegateBatchStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxRedelegateBatchStorageKeys::SourceAddress(entry) => {
                format!("source-{}-address", entry)
            }
            TxRedelegateBatchStorageKeys::DestValidatorAddress(entry) => {
                format!("validator-{}-address", entry)
            }
            TxRedelegateBatchStorageKeys::SourceValidatorAddress(entry) => {
                format!("source-validator-{}-address", entry)
            }
            TxRedelegateBatchStorageKeys::Amount(entry) => format!("amount-{}", entry),
            TxRedelegateBatchStorageKeys::BatchSize => "batch-size".to_string(),
            TxRedelegateBatchStorageKeys::AtomicBatch => "batch-atomic".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxRedelegateBatch {}

impl TxRedelegateBatch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxRedelegateBatch {
    type P = TxRedelegateBatchParameters;
    type B = namada_sdk::args::Redelegate;

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
            let validator_src = parameters.src_validators[param_idx]
                .to_namada_address(sdk)
                .await;
            let validator_target =
                if let Some(address) = parameters.dest_validators[param_idx].clone() {
                    address.to_namada_address(sdk).await
                } else {
                    return Ok(StepResult::no_op());
                };

            let bond_amount = Amount::from(parameters.amounts[param_idx]);

            let redelegate_tx_builder = sdk.namada.new_redelegation(
                source_address.clone(),
                validator_src.clone(),
                validator_target.clone(),
                bond_amount,
            );

            let redelegate_tx_builder = self
                .add_settings(sdk, redelegate_tx_builder, settings.clone())
                .await;

            let res = redelegate_tx_builder
                .build(&sdk.namada)
                .await
                .map_err(|e| TaskError::Build(e.to_string()));

            if res.is_ok() {
                storage.add(
                    TxRedelegateBatchStorageKeys::SourceAddress(param_idx).to_string(),
                    source_address.to_string(),
                );
                storage.add(
                    TxRedelegateBatchStorageKeys::SourceValidatorAddress(param_idx).to_string(),
                    validator_src.to_string(),
                );
                storage.add(
                    TxRedelegateBatchStorageKeys::DestValidatorAddress(param_idx).to_string(),
                    validator_target.to_string(),
                );
                storage.add(
                    TxRedelegateBatchStorageKeys::Amount(param_idx).to_string(),
                    bond_amount.raw_amount().to_string(),
                );
                txs.push(res);
            }
        }

        let txs = txs
            .into_iter()
            .filter_map(|res| res.ok())
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
            TxRedelegateBatchStorageKeys::BatchSize.to_string(),
            txs.len().to_string(),
        );
        storage.add(
            TxRedelegateBatchStorageKeys::AtomicBatch.to_string(),
            is_atomic.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxRedelegateBatchParametersDto {
    pub sources: Vec<Value>,
    pub src_validators: Vec<Value>,
    pub dest_validators: Vec<Value>,
    pub amounts: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxRedelegateBatchParameters {
    pub sources: Vec<AccountIndentifier>,
    pub src_validators: Vec<AccountIndentifier>,
    pub dest_validators: Vec<Option<AccountIndentifier>>,
    pub amounts: Vec<u64>,
}

impl TaskParam for TxRedelegateBatchParameters {
    type D = TxRedelegateBatchParametersDto;

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
                let src_validator = match dto.src_validators[i].clone() {
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
                let dest_validator = match dto.dest_validators[i].clone() {
                    Value::Ref { value, field } => {
                        let was_step_successful = state.is_step_successful(&value);
                        if !was_step_successful {
                            return None;
                        }
                        let data = state.get_step_item(&value, &field);
                        match field.to_lowercase().as_str() {
                            "alias" => Some(AccountIndentifier::Alias(data)),
                            "public-key" => Some(AccountIndentifier::PublicKey(data)),
                            "state" => Some(AccountIndentifier::StateAddress(state.get_address(&data))),
                            _ => Some(AccountIndentifier::Address(data)),
                        }
                    }
                    Value::Value { value } => {
                        if value.starts_with(ADDRESS_PREFIX) {
                            Some(AccountIndentifier::Address(value))
                        } else {
                            Some(AccountIndentifier::Alias(value))
                        }
                    }
                    Value::Fuzz { value } => {
                        let step_id = value.expect("Redelgate task requires fuzz for dest valdidator to define the step id to a validator query step");
                        let total_validators = state
                            .get_step_item(
                                &step_id,
                                ValidatorsQueryStorageKeys::TotalValidator
                                    .to_string()
                                    .as_str(),
                            )
                            .parse::<u64>()
                            .unwrap();

                        if total_validators < 2 {
                            None
                        } else {
                            loop {
                                let validator_idx = rand::thread_rng().gen_range(0..total_validators);
                                let validator_address = state.get_step_item(
                                    &step_id,
                                    ValidatorsQueryStorageKeys::Validator(validator_idx)
                                        .to_string()
                                        .as_str(),
                                );
                                let dest_validator = AccountIndentifier::Address(validator_address);

                                if dest_validator != src_validator {
                                    break Some(dest_validator);
                                }
                            }
                        }
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

                Some((source, src_validator, dest_validator, amount))
            })
            .collect::<Vec<(
                AccountIndentifier,
                AccountIndentifier,
                Option<AccountIndentifier>,
                u64,
            )>>();

        Some(Self {
            sources: batch.iter().map(|t| t.0.clone()).collect(),
            src_validators: batch.iter().map(|t| t.1.clone()).collect(),
            dest_validators: batch.iter().map(|t| t.2.clone()).collect(),
            amounts: batch.iter().map(|t| t.3).collect(),
        })
    }
}
