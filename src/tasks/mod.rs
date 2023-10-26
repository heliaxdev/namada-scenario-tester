use async_trait::async_trait;

use crate::{scenario::StepResult, state::state::Storage, sdk::namada::Sdk};

pub mod init_account;
pub mod tx_transparent_transfer;
pub mod wallet_new_key;

#[async_trait(?Send)]
pub trait Task {
    type P: TaskParam;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult;

    async fn run(&self, sdk: &Sdk, dto: <<Self as Task>::P as TaskParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(sdk, parameters, state).await
    }
}

pub trait TaskParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
