use async_trait::async_trait;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage};

pub mod epoch;
pub mod height;

#[async_trait(?Send)]
pub trait Wait {
    type P: WaitParam;

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, state: &Storage) -> StepResult;

    async fn run(
        &self,
        sdk: &Sdk,
        dto: <<Self as Wait>::P as WaitParam>::D,
        state: &Storage,
    ) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(sdk, parameters, state).await
    }
}

pub trait WaitParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
