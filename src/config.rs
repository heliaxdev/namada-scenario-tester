use clap::ArgAction;

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
    #[arg(required = true)]
    pub rpc: String,

    #[clap(long, env)]
    #[arg(required = true)]
    pub chain_id: String,

    #[clap(long, env)]
    #[arg(required = true)]
    pub faucet_sk: String,

    #[clap(long, env, default_value = "1")]
    pub runs: u64,

    #[clap(long, env, action=ArgAction::SetFalse)]
    pub dry_run: bool,
}
