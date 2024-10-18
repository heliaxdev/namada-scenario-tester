use std::time::Duration;

use async_trait::async_trait;
use namada_sdk::{rpc};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Wait, WaitParam};

#[derive(Clone, Debug, Default)]
pub struct HeightWait {}

impl HeightWait {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Wait for HeightWait {
    type P = HeightWaitParameters;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, _state: &Storage) -> StepResult {
        let start = paramaters.from;
        let to = paramaters.to;
        let r#for = paramaters.r#for;

        match (start, r#for, to) {
            (Some(start), Some(r#for), None) => {
                let to_block = start + r#for;

                loop {
                    let block = rpc::query_block(&sdk.namada.clone_client()).await;

                    let current_block = match block {
                        Ok(Some(height)) => height,
                        Ok(_) => return StepResult::fail("Block height is None".to_string()),
                        Err(e) => return StepResult::fail(e.to_string()),
                    };

                    if current_block.height.0 >= to_block {
                        break;
                    } else {
                        sleep(Duration::from_secs(10)).await
                    }
                }
            }
            (None, None, Some(to)) => loop {
                let block = rpc::query_block(&sdk.namada.clone_client()).await;

                let current_block = match block {
                    Ok(Some(height)) => height,
                    Ok(_) => return StepResult::fail("Block height is None".to_string()),
                    Err(e) => return StepResult::fail(e.to_string()),
                };

                if current_block.height.0 >= to {
                    break;
                } else {
                    sleep(Duration::from_secs(10)).await
                }
            },
            (_, _, _) => unimplemented!(),
        };

        StepResult::success_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeightWaitParametersDto {
    pub from: Option<Value>,
    pub r#for: Option<Value>,
    pub to: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct HeightWaitParameters {
    pub from: Option<u64>,
    pub r#for: Option<u64>,
    pub to: Option<u64>,
}

impl WaitParam for HeightWaitParameters {
    type D = HeightWaitParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let from = dto.from.map(|from| match from {
            Value::Ref { value, field } => {
                state.get_step_item(&value, &field).parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });
        let r#for = dto.r#for.map(|r#for| match r#for {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });
        let to = dto.to.map(|to| match to {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });

        Self { from, to, r#for }
    }
}
