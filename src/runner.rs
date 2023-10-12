use std::collections::HashMap;

use crate::scenario::{Scenario, StepData, StepOutcome};

#[derive(Clone, Debug, Default)]
pub struct Runner {
    results: HashMap<u64, StepOutcome>,
    states: HashMap<u64, StepData>
}

impl Runner {
    pub fn run(&mut self, scenario: Scenario) {
        scenario.steps.iter().for_each(|step| {
            let successful_prev_step = if step.id.eq(&0) {
                true   
            } else {
                self
                .results
                .get(&(step.id - 1))
                .map(|step| step.success)
                .unwrap_or(true)
            };
            
            if successful_prev_step {
                println!("Running step {}...", step.config);
                let result = step.run(&self.states);
                if result.is_succesful() {
                    println!("Step {} executed succesfully.", step.config);
                    self.results.insert(step.id, result.outcome);
                    self.states.insert(step.id, result.data);
                } else {
                    println!("Step {} errored bepbop.", step.config);
                    self.results.insert(step.id, result.outcome);
                    self.states.insert(step.id, result.data);
                }
            } else {
                return;
            }
        });
    }
}
