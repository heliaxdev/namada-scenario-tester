use std::{fs, io::Read};

use clap::Parser;
use namada_scenario_tester::{config::AppConfig, runner::Runner, scenario::Scenario};
use rand::Rng;

#[tokio::main]
async fn main() {
    let config = AppConfig::parse();

    let (scenario_json, scenario_path) = if let Some(scenario) = config.scenario.clone() {
        (fs::read_to_string(&scenario).unwrap(), scenario)
    } else {
        let mut paths = fs::read_dir("../scenarios").unwrap();
        let scenario_index = rand::thread_rng().gen_range(0..paths.size_hint().0);

        let scenario_file_path = paths.nth(scenario_index).unwrap().unwrap().path();

        let mut file = fs::File::open(&scenario_file_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        (content, scenario_file_path.to_string_lossy().to_string())
    };
    let scenario: Scenario = serde_json::from_str(&scenario_json).unwrap();

    Runner::default()
        .run(scenario, &config, scenario_path)
        .await;
}
