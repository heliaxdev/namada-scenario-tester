use std::fs;

use clap::Parser;
use namada_load_tester::{config::AppConfig, runner::Runner, scenario::Scenario};

fn main() {
    let config = AppConfig::parse();

    let yaml = fs::read_to_string(config.scenario).unwrap();
    let scenario: Scenario = serde_json::from_str(&yaml).unwrap();

    Runner::default().run(scenario);
}
