use async_trait::async_trait;

use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage};

pub mod balance;
pub mod bonds;
pub mod reveal_pk;
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
        avoid_check: bool,
    ) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        let outcome = self.execute(sdk, parameters, state).await;

        if avoid_check {
            return StepResult::skip_check(outcome.is_succesful());
        } else {
            return outcome;
        }
    }
}

pub trait CheckParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
