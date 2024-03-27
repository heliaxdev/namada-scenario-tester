use std::time::Duration;

use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Wait, WaitParam};

#[derive(Clone, Debug, Default)]
pub struct EpochWait {}

impl EpochWait {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Wait for EpochWait {
    type P = EpochWaitParameters;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, _state: &Storage) -> StepResult {
        let start = paramaters.from;
        let to = paramaters.to;
        let r#for = paramaters.r#for;

        match (start, r#for, to) {
            (Some(start), Some(r#for), None) => {
                let epoch = rpc::query_epoch(sdk.namada.client()).await;

                let _current_epoch = if let Ok(epoch) = epoch {
                    epoch
                } else {
                    return StepResult::fail();
                };

                let to_epoch = start + r#for;

                loop {
                    let epoch = rpc::query_epoch(sdk.namada.client()).await;

                    let current_epoch = if let Ok(epoch) = epoch {
                        epoch
                    } else {
                        return StepResult::fail();
                    };

                    if current_epoch.0 >= to_epoch {
                        break;
                    } else {
                        sleep(Duration::from_secs(10)).await
                    }
                }
            }
            (None, None, Some(to)) => loop {
                let epoch = rpc::query_epoch(sdk.namada.client()).await;

                let current_epoch = if let Ok(epoch) = epoch {
                    epoch
                } else {
                    return StepResult::fail();
                };

                if current_epoch.0 >= to {
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
            Value::Ref { value, field } => {
                state.get_step_item(&value, &field).parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });

        Self { from, to, r#for }
    }
}
