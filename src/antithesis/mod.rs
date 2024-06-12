use std::collections::HashMap;

use antithesis_sdk::assert_sometimes;
use serde::Serialize;
use serde_json::json;

use crate::scenario::StepType;

#[derive(Clone, Debug, Default, Serialize)]
pub struct AntithesisReport {
    pub successful: HashMap<String, u64>,
    pub failed: HashMap<String, u64>,
    pub enabled: bool
}

impl AntithesisReport {
    pub fn add_successful(&mut self, step_type: StepType) {
        self.successful.entry(step_type.to_string()).and_modify(|counter| { *counter += 1 }).or_insert(1);
    }

    pub fn add_failed(&mut self, step_type: StepType) {
        self.failed.entry(step_type.to_string()).and_modify(|counter| { *counter += 1 }).or_insert(1);
    }

    pub fn total_steps(&self) -> usize {
        self.successful.len() + self.failed.len()
    }
}