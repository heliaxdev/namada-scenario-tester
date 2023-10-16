use crate::{config::AppConfig, scenario::Scenario, state::state::Storage};

#[derive(Clone, Debug, Default)]
pub struct Runner {
    storage: Storage,
}

impl Runner {
    pub fn run(&mut self, scenario: Scenario, config: &AppConfig) {
        for _ in 0..config.runs {
            scenario.steps.iter().for_each(|step| {
                let successful_prev_step = if step.id.eq(&0) {
                    true
                } else {
                    self.storage.is_step_successful(&(step.id - 1))
                };

                if successful_prev_step {
                    println!("Running step {}...", step.config);
                    println!("{:?}", step.settings);
                    let result = step.run(&self.storage, &config.rpcs, &config.chain_id);
                    if result.is_succesful() {
                        println!("Step {} executed succesfully.", step.config);
                        self.storage.save_step_result(step.id, result)
                    } else {
                        println!("Step {} errored bepbop.", step.config);
                        self.storage.save_step_result(step.id, result)
                    }
                }
            });
        }
    }
}
