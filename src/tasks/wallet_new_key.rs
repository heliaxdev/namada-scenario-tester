use std::collections::HashMap;

use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::scenario::{StepData, StepOutcome, StepResult};

use super::{Task, TaskParam};

#[derive(Clone, Debug)]
pub struct WalletNewKey {
    pub parameters: WalletNewKeyParameters,
}

impl Task for WalletNewKey {
    type P = WalletNewKeyParameters;

    fn run(
        dto: <<Self as Task>::P as TaskParam>::D,
        state: &HashMap<u64, StepData>,
    ) -> StepResult {
        let _parameters = Self::P::from_dto(dto, state);

        // run
        let random_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let random_alias = format!("lt-{}", random_str);
        println!("namadaw address gen --alias {} --unsafe-dont-encrypt", random_alias);

        let mut data = HashMap::new();
        data.insert("key-alias".to_string(), random_alias.to_string());

        StepResult {
            outcome: StepOutcome::successful(),
            data: StepData { data },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalletNewKeyParametersDto {}

impl WalletNewKeyParametersDto {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
pub struct WalletNewKeyParameters {}

impl TaskParam for WalletNewKeyParameters {
    type D = WalletNewKeyParametersDto;

    fn from_dto(_dto: Self::D, _state: &HashMap<u64, StepData>) -> Self {
        WalletNewKeyParameters {}
    }
}
