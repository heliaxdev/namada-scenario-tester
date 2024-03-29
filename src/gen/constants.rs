use namada_sdk::token::NATIVE_SCALE;

pub const PROPOSAL_FUNDS: u64 = 500 * NATIVE_SCALE;
pub const MIN_FEE: u64 = 5 * NATIVE_SCALE;
pub const VALIDATOR_ZERO_STORAGE_KEY: &str = "validator-0-address";
pub const BOND_VALIDATOR_STORAGE_KEY: &str = "validator-address";
pub const UNBOND_VALIDATOR_STORAGE_KEY: &str = "validator-address";
