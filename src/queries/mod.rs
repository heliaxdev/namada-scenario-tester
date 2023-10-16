use crate::{scenario::StepResult, state::state::Storage};

pub mod account;
pub mod balance;

pub trait Query {
    type P: QueryParam;

    fn execute(&self, paramaters: Self::P, state: &Storage) -> StepResult;

    fn run(&self, dto: <<Self as Query>::P as QueryParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(parameters, state)
    }
}

pub trait QueryParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
