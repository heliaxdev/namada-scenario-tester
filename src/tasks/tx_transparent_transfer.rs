use std::str::FromStr;

use async_trait::async_trait;
use namada_sdk::{
    args::InputAmount,
    core::types::{
        address::Address as NamadaAddress,
        masp::{TransferSource, TransferTarget},
        token::{self, DenominatedAmount},
    },
    Namada, rpc,
};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct TxTransparentTransfer {
    rpc: String,
    chain_id: String,
}

impl TxTransparentTransfer {
    pub fn new(sdk: &Sdk) -> Self {
        Self {
            rpc: sdk.rpc.clone(),
            chain_id: sdk.chain_id.clone(),
        }
    }
}

#[async_trait(?Send)]
impl Task for TxTransparentTransfer {
    type P = TxTransparentTransferParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let nam_token_address = sdk.namada.native_token();

        let mut transfer_tx_builder = sdk.namada.new_transfer(
            TransferSource::Address(NamadaAddress::decode(parameters.source.address).unwrap()),
            TransferTarget::Address(NamadaAddress::decode(parameters.target.address).unwrap()),
            nam_token_address,
            InputAmount::Unvalidated(DenominatedAmount::native(token::Amount::from_u64(
                parameters.amount,
            ))),
        );

        let (mut transfer_tx, signing_data, _epoch) = transfer_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");
        sdk.namada
            .sign(&mut transfer_tx, &transfer_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign reveal pk tx");
        let tx = sdk.namada.submit(transfer_tx, &transfer_tx_builder.tx).await;

        let mut storage = StepStorage::default();
        storage.add("amount".to_string(), parameters.amount.to_string());
        storage.add("token".to_string(), parameters.token.to_string());

        self.fetch_info(&sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxTransparentTransferParametersDto {
    source: Value,
    target: Value,
    amount: Value,
    token: Value,
}

#[derive(Clone, Debug)]
pub struct TxTransparentTransferParameters {
    source: Address,
    target: Address,
    amount: u64,
    token: String,
}

impl TaskParam for TxTransparentTransferParameters {
    type D = TxTransparentTransferParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let target = match dto.target {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let amount = match dto.amount {
            Value::Ref { value } => state
                .get_step_item(&value, "amount")
                .parse::<u64>()
                .unwrap(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value } => state.get_step_item(&value, "token-address"),
            Value::Value { value } => value,
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            source,
            target,
            amount,
            token,
        }
    }
}
