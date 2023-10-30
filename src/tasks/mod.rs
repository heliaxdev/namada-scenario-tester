use async_trait::async_trait;
use namada_sdk::{rpc, Namada};

use crate::{scenario::StepResult, state::state::{Storage, StepStorage}, sdk::namada::Sdk};

pub mod init_account;
pub mod tx_transparent_transfer;
pub mod wallet_new_key;

#[async_trait(?Send)]
pub trait Task {
    type P: TaskParam;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult;

    async fn fetch_info(&self, sdk: &Sdk, step_storage: &mut StepStorage) {
        let block = rpc::query_block(sdk.namada.client()).await.unwrap().unwrap();
        let epoch = rpc::query_epoch(sdk.namada.client()).await.unwrap();

        step_storage.add("epoch".to_string(), "10".to_string());
        step_storage.add("height".to_string(), "10".to_string());
    }

    async fn run(&self, sdk: &Sdk, dto: <<Self as Task>::P as TaskParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(sdk, parameters, state).await
    }
}

pub trait TaskParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
