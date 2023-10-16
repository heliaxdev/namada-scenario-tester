use serde::Deserialize;

use crate::{
    scenario::StepResult,
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
    pub fn new(rpc: String, chain_id: String) -> Self {
        Self { rpc, chain_id }
    }
}

impl Task for TxTransparentTransfer {
    type P = TxTransparentTransferParameters;

    fn execute(&self, parameters: Self::P, _state: &Storage) -> StepResult {
        println!(
            "namadac transfer --source {} --target {} --amount {} --token {} --signing-keys {:?}, --node {}",
            parameters.source.alias,
            parameters.target.alias,
            parameters.amount,
            parameters.token,
            parameters.source.keys,
            format!("{}/{}", self.rpc, self.chain_id)
        );

        let mut storage = StepStorage::default();
        storage.add("amount".to_string(), parameters.amount.to_string());
        storage.add("token".to_string(), parameters.token.to_string());
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

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
