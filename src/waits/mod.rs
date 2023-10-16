use crate::{scenario::StepResult, state::state::Storage};

pub mod epoch;
pub mod height;

pub trait Wait {
    type P: WaitParam;

    fn execute(&self, paramaters: Self::P, state: &Storage) -> StepResult;

    fn run(&self, dto: <<Self as Wait>::P as WaitParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(parameters, state)
    }
}

pub trait WaitParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
