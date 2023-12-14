use markdown_gen::markdown::{AsMarkdown, List, Markdown};

use crate::{config::AppConfig, scenario::Scenario, state::state::Storage};

pub struct Report {
    pub config: AppConfig,
    pub storage: Storage,
    pub scenario: Scenario,
}

impl Report {
    pub fn new(config: &AppConfig, storage: Storage, scenario: Scenario) -> Self {
        Self {
            config: config.clone(),
            storage,
            scenario,
        }
    }
    pub fn generate_report(&self) -> String {
        let mut md = Markdown::new(Vec::new());

        md.write("Info".heading(2)).unwrap();

        let outcome = self.storage.is_succesful().to_string();
        let info_list = List::new(false)
            .item("Chain ID: ".paragraph().append(self.config.chain_id.code()))
            .item("RPC url: ".paragraph().append(self.config.rpc.code()))
            .item("Outcome: ".paragraph().append(outcome.code()))
            .item("Scenario: ".paragraph().append(self.config.scenario.code()));

        md.write(info_list).unwrap();
        md.write("\n").unwrap();

        md.write("Steps".heading(2)).unwrap();
        for (index, (id, step_outcome)) in self.storage.step_results.iter().enumerate() {
            let step = self.scenario.steps.get(index).unwrap();
            let step_type = step.config.to_string();
            let outcome = step_outcome.to_string();
            let step_id = id.to_string();
            let step_list = List::new(false)
                .title("Step id: ".paragraph().append(step_id.code()))
                .item("Type: ".paragraph().append(step_type.code()))
                .item("Outcome: ".paragraph().append(outcome.code()));

            md.write(step_list).unwrap();
            md.write("\n").unwrap();
        }

        let vec = md.into_inner();
        String::from_utf8(vec).unwrap()
    }
}
