use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::args::Bond;
use serde::{Deserialize, Serialize};

use crate::{sdk::namada::Sdk, state::state::Storage, utils::settings::TxSettings};

use super::{
    bond::{TxBondParameters, TxBondParametersDto},
    tx_transparent_transfer::{
        TxTransparentTransferParameters, TxTransparentTransferParametersDto,
    },
    BuildResult, Task, TaskError, TaskParam,
};

pub enum TxBatchStorageKeys {
    SourceAddress,
    ValidatorAddress,
    Amount,
}

impl ToString for TxBatchStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxBatchStorageKeys::SourceAddress => "source-address".to_string(),
            TxBatchStorageKeys::ValidatorAddress => "validator-address".to_string(),
            TxBatchStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxBatch {}

impl TxBatch {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxBatch {
    type P = TxBatchParameters;
    type B = Bond;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        todo!();
    }
}

impl Display for TxBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-batch")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BatchParameterDto {
    TransparentTransfer(TxTransparentTransferParametersDto),
    Bond(TxBondParametersDto),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxBatchParametersDto {
    pub batch: Vec<BatchParameterDto>,
}

#[derive(Clone, Debug)]
pub enum BatchParameters {
    TransparentTransfer(TxTransparentTransferParameters),
    Bond(TxBondParameters),
}

#[derive(Clone, Debug)]
pub struct TxBatchParameters {
    batch: Vec<BatchParameters>,
}

impl TaskParam for TxBatchParameters {
    type D = TxBatchParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let batch = dto
            .batch
            .into_iter()
            .map(|dto| match dto {
                BatchParameterDto::TransparentTransfer(dto) => {
                    BatchParameters::TransparentTransfer(
                        TxTransparentTransferParameters::parameter_from_dto(dto, state)
                            .expect("Should be able to create batch parameters"),
                    )
                }
                BatchParameterDto::Bond(dto) => BatchParameters::Bond(
                    TxBondParameters::parameter_from_dto(dto, state)
                        .expect("Should be able to create batch parameters"),
                ),
            })
            .collect();

        Some(Self { batch })
    }
}
