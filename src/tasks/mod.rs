use crate::{scenario::StepResult, state::state::Storage};

pub mod init_account;
pub mod tx_transparent_transfer;
pub mod wallet_new_key;

pub trait Task {
    type P: TaskParam;

    fn execute(&self, paramaters: Self::P, state: &Storage) -> StepResult;

    fn run(&self, dto: <<Self as Task>::P as TaskParam>::D, state: &Storage) -> StepResult {
        let parameters = Self::P::from_dto(dto, state);

        self.execute(parameters, state)
    }
}

pub trait TaskParam {
    type D;

    fn from_dto(dto: Self::D, state: &Storage) -> Self;
}
