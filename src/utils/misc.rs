

use std::fmt::Display;

use namada_sdk::proof_of_stake::types::ValidatorState as NamadaValidatorState;

#[derive(Debug, Clone)]
pub enum ValidatorState {
    Consensus,
    BelowCapacity,
    BelowThreshold,
    Inactive,
    Jailed,
    Unknown
}

impl Display for ValidatorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidatorState::Consensus => write!(f, "consensus"),
            ValidatorState::BelowCapacity => write!(f, "below-capacity"),
            ValidatorState::BelowThreshold => write!(f, "below-threshold"),
            ValidatorState::Inactive => write!(f, "inactive"),
            ValidatorState::Jailed => write!(f, "wallet-new-key"),
            ValidatorState::Unknown => write!(f, "wallet-new-key"),
        }
    }
}

impl From<NamadaValidatorState> for ValidatorState {
    fn from(value: NamadaValidatorState) -> Self {
        match value {
            NamadaValidatorState::Consensus => ValidatorState::Consensus,
            NamadaValidatorState::BelowCapacity => ValidatorState::BelowCapacity,
            NamadaValidatorState::BelowThreshold => ValidatorState::BelowThreshold,
            NamadaValidatorState::Inactive => ValidatorState::Inactive,
            NamadaValidatorState::Jailed => ValidatorState::Jailed,
        }
    }
}