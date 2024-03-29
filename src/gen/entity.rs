use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::utils::{settings::TxSettingsDto, value::Value};

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Alias {
    inner: String,
}

impl From<String> for Alias {
    fn from(value: String) -> Self {
        Alias { inner: value }
    }
}

impl Alias {
    pub fn native_token() -> Self {
        Self {
            inner: "nam".to_string(),
        }
    }
}

impl Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum AddressType {
    Enstablished,
    #[default]
    Implicit,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Account {
    pub threshold: u64,
    pub implicit_addresses: Vec<Alias>,
    pub address_type: AddressType,
    pub alias: Alias,
    pub is_validator: bool,
}

impl Account {
    pub fn new(
        alias: Alias,
        implicit_addresses: Vec<Alias>,
        address_type: AddressType,
        threshold: u64,
        is_validator: bool,
    ) -> Self {
        Self {
            threshold,
            implicit_addresses,
            address_type,
            alias,
            is_validator,
        }
    }

    pub fn new_implicit_address(alias: Alias) -> Self {
        Self::new(alias.clone(), vec![alias], AddressType::Implicit, 1, false)
    }

    pub fn new_enstablished_address(alias: Alias, pks: Vec<Alias>, threshold: u64) -> Self {
        Self::new(
            alias.clone(),
            pks,
            AddressType::Enstablished,
            threshold,
            false,
        )
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Bond {
    pub source: Alias,
    pub amount: u64,
    pub step_id: u64,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Unbond {
    pub source: Alias,
    pub amount: u64,
    pub step_id: u64,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Builder)]
pub struct TxSettings {
    pub signers: Vec<Alias>,
    pub broadcast_only: bool,
}

impl From<&TxSettings> for TxSettingsDto {
    fn from(value: &TxSettings) -> Self {
        Self {
            broadcast_only: Some(value.broadcast_only),
            gas_token: None,
            gas_payer: None,
            signers: Some(
                value
                    .signers
                    .iter()
                    .map(|signer| Value::Value {
                        value: signer.to_string(),
                    })
                    .collect(),
            ),
            expiration: None,
            gas_limit: None,
        }
    }
}

impl TxSettings {
    pub fn from_signers(signers: Vec<Alias>) -> Self {
        TxSettings {
            signers,
            broadcast_only: false,
        }
    }
}
