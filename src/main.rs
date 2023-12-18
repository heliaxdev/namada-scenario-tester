use std::fs;

use clap::Parser;
use namada_scenario_tester::{config::AppConfig, runner::Runner, scenario::Scenario};

#[tokio::main]
async fn main() {
    let config = AppConfig::parse();

    let json_str = fs::read_to_string(&config.scenario).unwrap();
    let scenario: Scenario = serde_json::from_str(&json_str).unwrap();

    Runner::default().run(scenario, &config).await;
}
