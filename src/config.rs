#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum CargoEnv {
    Development,
    Production,
}

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env, value_enum)]
    pub cargo_env: CargoEnv,

    #[clap(long, env)]
    pub scenario: String,

    #[clap(long, env)]
    #[arg(required=true, num_args=1..)]
    pub rpcs: Vec<String>,

    #[clap(long, env)]
    #[arg(required = true)]
    pub chain_id: String,

    #[clap(long, env, default_value = "1")]
    pub runs: u64,
}
