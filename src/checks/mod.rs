use async_trait::async_trait;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage};

pub mod balance;
pub mod bonds;
pub mod step;
pub mod storage;

#[async_trait(?Send)]
pub trait Check {
    type P: CheckParam;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult;

    async fn run(
        &self,
        sdk: &Sdk,
        dto: <<Self as Check>::P as CheckParam>::D,
        state: &Storage,
    ) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(sdk, parameters, state).await
    }
}

pub trait CheckParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
