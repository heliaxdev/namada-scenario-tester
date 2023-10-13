use std::collections::HashMap;

use crate::scenario::StepResult;

#[derive(Clone, Debug, Default)]
pub struct StepOutcome {
    success: bool,
}

impl StepOutcome {
    pub fn is_succesful(&self) -> bool {
        self.success
    }

    pub fn success() -> Self {
        Self { success: true }
    }

    pub fn fail() -> Self {
        Self { success: false }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StepStorage {
    storage: HashMap<String, String>,
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
pub enum AddressType {
    Implicit,
    Enstablished,
}

#[derive(Clone, Debug)]
pub struct Address {
    pub alias: String,
    pub address: String,
    pub keys: Vec<String>,
    pub threshold: u64,
    pub address_type: AddressType,
}

impl Address {
    pub fn new(alias: String, address: String, keys: Vec<String>, threshold: u64) -> Self {
        Self {
            alias,
            address,
            keys,
            threshold,
            address_type: AddressType::Enstablished,
        }
    }

    pub fn from_alias(alias: String) -> Self {
        Self {
            alias: alias.clone(),
            address: alias.clone(),
            keys: vec![alias],
            threshold: 1,
            address_type: AddressType::Implicit,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Storage {
    pub step_results: HashMap<u64, StepOutcome>,
    pub step_states: HashMap<u64, StepStorage>,
    pub accounts: HashMap<String, Address>,
}

impl Storage {
    pub fn save_step_outcome(&mut self, step_id: u64, step_outcome: StepOutcome) {
        self.step_results.insert(step_id, step_outcome);
    }

    pub fn save_step_state(&mut self, step_id: u64, step_state: StepStorage) {
        self.step_states.insert(step_id, step_state);
    }

    pub fn save_account(&mut self, account: Address) {
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
            .expect("Step id shoudl exist.")
            .success
    }

    pub fn save_step_result(&mut self, step_id: u64, step_result: StepResult) {
        self.save_step_outcome(step_id, step_result.outcome);
        self.save_step_state(step_id, step_result.data);
        for account in step_result.accounts {
            self.save_account(account);
        }
    }

    pub fn get_address(&self, alias: &str) -> Address {
        self.accounts
            .get(alias)
            .expect("Address should be there")
            .to_owned()
    }
}
