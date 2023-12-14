use std::{collections::HashMap, fmt::Display};

use crate::scenario::StepResult;

#[derive(Clone, Debug, Default)]
pub enum StepOutcome {
    #[default]
    SUCCESS,
    FAIL,
    CHECK_FAIL,
}

impl Display for StepOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepOutcome::SUCCESS => write!(f, "success"),
            StepOutcome::FAIL => write!(f, "fail"),
            StepOutcome::CHECK_FAIL => write!(f, "check fail"),
        }
    }
}

impl StepOutcome {
    pub fn is_succesful(&self) -> bool {
        matches!(self, Self::SUCCESS)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::FAIL)
    }

    pub fn success() -> Self {
        Self::SUCCESS
    }

    pub fn fail() -> Self {
        Self::FAIL
    }

    pub fn check_fail() -> Self {
        Self::CHECK_FAIL
    }
}

#[derive(Clone, Debug, Default)]
pub struct StepStorage {
    pub storage: HashMap<String, String>,
}

impl StepStorage {
    pub fn add(&mut self, key: String, value: String) {
        self.storage.insert(key, value);
    }

    pub fn get_field(&self, field: &str) -> String {
        self.storage
            .get(field)
            .expect("Field should be present in data.")
            .to_owned()
    }
}

#[derive(Clone, Debug)]
pub enum StateAddressType {
    Implicit,
    Enstablished,
}

#[derive(Clone, Debug)]
pub struct StateAddress {
    pub alias: String,
    pub address: String,
    pub keys: Vec<String>,
    pub threshold: u64,
    pub address_type: StateAddressType,
}

impl StateAddress {
    pub fn new(
        alias: String,
        address: String,
        keys: Vec<String>,
        threshold: u64,
        address_type: StateAddressType,
    ) -> Self {
        Self {
            alias,
            address,
            keys,
            threshold,
            address_type,
        }
    }

    pub fn new_enstablished(
        alias: String,
        address: String,
        keys: Vec<String>,
        threshold: u64,
    ) -> Self {
        Self {
            alias,
            address,
            keys,
            threshold,
            address_type: StateAddressType::Enstablished,
        }
    }

    pub fn new_implicit(alias: String, address: String) -> Self {
        Self {
            alias: alias.clone(),
            address,
            keys: vec![alias],
            threshold: 1,
            address_type: StateAddressType::Implicit,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Storage {
    pub step_results: HashMap<u64, StepOutcome>,
    pub step_states: HashMap<u64, StepStorage>,
    pub accounts: HashMap<String, StateAddress>,
}

impl Storage {
    pub fn is_succesful(&self) -> StepOutcome {
        let outcome = self
            .step_results
            .values()
            .fold(true, |acc, e| acc && e.is_succesful());

        if outcome {
            StepOutcome::SUCCESS
        } else {
            StepOutcome::FAIL
        }
    }

    pub fn save_step_outcome(&mut self, step_id: u64, step_outcome: StepOutcome) {
        self.step_results.insert(step_id, step_outcome);
    }

    pub fn save_step_state(&mut self, step_id: u64, step_state: StepStorage) {
        self.step_states.insert(step_id, step_state);
    }

    pub fn save_account(&mut self, account: StateAddress) {
        self.accounts.insert(account.alias.clone(), account);
    }

    pub fn get_step_item(&self, step_id: &u64, field: &str) -> String {
        self.step_states
            .get(step_id)
            .expect("Step id should be there.")
            .get_field(field)
    }

    pub fn is_step_successful(&self, step_id: &u64) -> bool {
        self.step_results
            .get(step_id)
            .expect("Step id should exist.")
            .is_succesful()
    }

    pub fn save_step_result(&mut self, step_id: u64, step_result: StepResult) {
        self.save_step_outcome(step_id, step_result.outcome);
        self.save_step_state(step_id, step_result.data);
        for account in step_result.accounts {
            self.save_account(account);
        }
    }

    pub fn get_last_epoch(&self) -> u64 {
        self.step_states
            .iter()
            .fold(0, |max, (_step_id, step_storage)| {
                if let Some(epoch) = step_storage.storage.get("epoch") {
                    if let Ok(epoch) = epoch.parse::<u64>() {
                        if max < epoch {
                            epoch
                        } else {
                            max
                        }
                    } else {
                        max
                    }
                } else {
                    max
                }
            })
    }

    pub fn get_address(&self, alias: &str) -> StateAddress {
        self.accounts
            .get(alias)
            .expect("Address should be there")
            .to_owned()
    }
}
