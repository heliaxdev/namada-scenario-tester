use std::collections::HashMap;

use crate::scenario::{StepData, StepResult};

pub mod init_account;
pub mod tx_transfer;
pub mod wallet_new_key;

pub trait Task {
    type P: TaskParam;

    fn run(dto: <<Self as Task>::P as TaskParam>::D, state: &HashMap<u64, StepData>) -> StepResult;
}

pub trait TaskParam {
    type D;
    
    fn from_dto(dto: Self::D, state: &HashMap<u64, StepData>) -> Self;
}