#[derive(clap::Parser, Clone)]
pub struct AppConfig {
    #[clap(long, env)]
    #[arg(required = true)]
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

    #[clap(long, env)]
    pub report_url: Option<String>,

    #[clap(long, env)]
    pub sha: Option<String>,

    #[clap(long, env)]
    pub minio_url: Option<String>,

    #[clap(long, env)]
    pub minio_access_key: Option<String>,

    #[clap(long, env)]
    pub minio_secret_key: Option<String>,

    #[clap(long, env)]
    pub artifacts_url: Option<String>,
}
