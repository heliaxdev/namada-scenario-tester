use async_trait::async_trait;

use crate::{scenario::StepResult, state::state::Storage, sdk::namada::Sdk};

pub mod account;
pub mod balance;

#[async_trait(?Send)]
pub trait Query {
    type P: QueryParam;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult;

    async fn run(&self, sdk: &Sdk, dto: <<Self as Query>::P as QueryParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(sdk, parameters, state).await
    }
}

pub trait QueryParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
