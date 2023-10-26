use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{StepStorage, Storage},
    utils::value::Value, sdk::namada::Sdk,
};

use super::{Wait, WaitParam};

#[derive(Clone, Debug, Default)]
pub struct EpochWait {
    rpc: String,
    chain_id: String,
}

impl EpochWait {
    pub fn new(sdk: &Sdk) -> Self {
        Self {
            rpc: sdk.rpc.clone(),
            chain_id: sdk.chain_id.clone(),
        }
    }
}

impl Wait for EpochWait {
    type P = EpochWaitParameters;

    fn execute(&self, paramaters: Self::P, _state: &Storage) -> StepResult {
        let start = paramaters.from;
        let to = paramaters.to;
        let r#for = paramaters.r#for;

        match (start, r#for, to) {
            (Some(start), Some(r#for), None) => {
                for _i in start..=start + r#for {
                    println!("namada client epoch");
                }
            }
            (None, None, Some(_to)) => {
                println!("namada client epoch");
            }
            (_, _, _) => unimplemented!(),
        };

        let mut storage = StepStorage::default();
        storage.add("epoch".to_string(), "10".to_string());
        storage.add("height".to_string(), "10".to_string());

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EpochWaitParametersDto {
    pub from: Option<Value>,
    pub r#for: Option<Value>,
    pub to: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct EpochWaitParameters {
    pub from: Option<u64>,
    pub r#for: Option<u64>,
    pub to: Option<u64>,
}

impl WaitParam for EpochWaitParameters {
    type D = EpochWaitParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let from = dto.from.map(|from| match from {
            Value::Ref { value } => state.get_step_item(&value, "epoch").parse::<u64>().unwrap(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });
        let r#for = dto.r#for.map(|r#for| match r#for {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });
        let to = dto.to.map(|to| match to {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });

        Self { from, to, r#for }
    }
}
