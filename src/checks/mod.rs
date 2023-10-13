use crate::{scenario::StepResult, state::state::Storage};

pub mod balance;
pub mod tx;

pub trait Check {
    type P: CheckParam;

    fn execute(&self, paramaters: Self::P, state: &Storage) -> StepResult;

    fn run(&self, dto: <<Self as Check>::P as CheckParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(parameters, state)
    }
}

pub trait CheckParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
