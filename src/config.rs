#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum CargoEnv {
    Development,
    Production,
}

#[derive(clap::Parser, Clone)]
pub struct AppConfig {
    #[clap(long, env, value_enum)]
    pub cargo_env: CargoEnv,

    #[clap(long, env)]
    pub scenario: String,

    #[clap(long, env)]
    #[arg(required = true)]
    pub rpc: String,

    #[clap(long, env)]
    #[arg(required = true)]
    pub chain_id: String,

    #[clap(long, env)]
    #[arg(required = true)]
    pub faucet_sk: String,
}
