use clap::Parser;
use namada_scenario_tester::{config::AppConfig, runner::Runner, scenario::Scenario};
use rand::Rng;
use std::{env, fs, io::Read, path::PathBuf};

#[tokio::main]
async fn main() {
    let config = AppConfig::parse();

    let mut workers = vec![];
    for worker_id in 0..config.workers {
        workers.push(async move { run(worker_id).await });
    }

    futures::future::join_all(workers).await;
}

async fn run(worker_id: u64) {
    let config = AppConfig::parse();

    let (scenario_json, scenario_path) = if let Some(scenario) = config.scenario.clone() {
        (fs::read_to_string(&scenario).unwrap(), scenario)
    } else {
        let paths = fs::read_dir("scenarios")
            .unwrap()
            .filter_map(|res| {
                let file = res.unwrap();
                if file.file_type().unwrap().is_file()
                    && file.path().extension().is_some()
                    && file
                        .path()
                        .extension()
                        .unwrap()
                        .eq_ignore_ascii_case("json")
                {
                    Some(file.path())
                } else {
                    None
                }
            })
            .collect::<Vec<PathBuf>>();
        let scenario_index = rand::thread_rng().gen_range(0..paths.len());

        let scenario_file_path = paths.get(scenario_index).unwrap().clone();

        let mut file = fs::File::open(&scenario_file_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        let is_antithesis_run = env::var("ANTITHESIS_OUTPUT_DIR");
        if let Ok(folder) = is_antithesis_run {
            let file_name = scenario_file_path.file_name().unwrap().to_string_lossy();
            let output_path = format!("{}/{}-{}.json", folder, file_name, worker_id);
            fs::copy(scenario_file_path.clone(), output_path).unwrap();
        }

        (content, scenario_file_path.to_string_lossy().to_string())
    };
    let scenario: Scenario = serde_json::from_str(&scenario_json).unwrap();

    Runner::default()
        .run(worker_id, scenario, &config, scenario_path)
        .await;
}
