use namada_sdk::token::NATIVE_SCALE;

pub const PROPOSAL_FUNDS: u64 = 5000 * NATIVE_SCALE;
pub const MIN_FEE: u64 = 250_000;
pub const DEFAULT_GAS_LIMIT: u64 = MIN_FEE;
pub const DEFAULT_GAS_PRICE: f64 = 0.000001;
pub const VALIDATOR_ZERO_STORAGE_KEY: &str = "validator-0-address";
pub const BOND_VALIDATOR_STORAGE_KEY: &str = "validator-address";
pub const UNBOND_VALIDATOR_STORAGE_KEY: &str = "validator-address";
pub const MAX_PGF_ACTIONS: u64 = 15;
pub const MAX_BATCH_SIZE: u64 = 5;
pub const MAX_BATCH_GAS_LIMIT: f64 = (MIN_FEE * MAX_BATCH_SIZE) as f64;
