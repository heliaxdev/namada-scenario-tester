use std::{collections::BTreeSet, fmt::Display};

use derive_builder::Builder;
use namada_scenario_tester::utils::{settings::TxSettingsDto, value::Value};

use crate::constants::DEFAULT_GAS_LIMIT;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn is_implicit(&self) -> bool {
        !self.inner.starts_with("load-tester-enst")
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

impl AddressType {
    pub fn is_implicit(&self) -> bool {
        matches!(self, AddressType::Implicit)
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Account {
    pub threshold: u64,
    pub implicit_addresses: BTreeSet<Alias>,
    pub address_type: AddressType,
    pub alias: Alias,
    pub is_validator: bool,
    pub is_active: bool,
}

impl Account {
    pub fn new(
        alias: Alias,
        implicit_addresses: BTreeSet<Alias>,
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
            is_active: true,
        }
    }

    pub fn new_implicit_address(alias: Alias) -> Self {
        Self::new(
            alias.clone(),
            BTreeSet::from_iter(vec![alias]),
            AddressType::Implicit,
            1,
            false,
        )
    }

    pub fn new_enstablished_address(alias: Alias, pks: BTreeSet<Alias>, threshold: u64) -> Self {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, Builder)]
pub struct TxSettings {
    pub signers: BTreeSet<Alias>,
    pub broadcast_only: bool,
    pub gas_limit: u64,
    pub gas_payer: Alias,
}

impl Default for TxSettings {
    fn default() -> Self {
        Self {
            signers: Default::default(),
            broadcast_only: Default::default(),
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_payer: Default::default(),
        }
    }
}

impl From<TxSettings> for TxSettingsDto {
    fn from(value: TxSettings) -> Self {
        Self {
            broadcast_only: Some(value.broadcast_only),
            gas_token: None,
            gas_payer: Some(Value::v(value.gas_payer.to_string())),
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
            gas_limit: Some(Value::v(value.gas_limit.to_string())),
        }
    }
}

impl TxSettings {
    pub fn new(
        signers: BTreeSet<Alias>,
        gas_payer: Alias,
        gas_limit: u64,
        broadcast_only: bool,
    ) -> Self {
        Self {
            signers,
            broadcast_only,
            gas_limit,
            gas_payer,
        }
    }

    pub fn default_from_implicit(signer: Alias) -> Self {
        Self {
            signers: BTreeSet::from_iter(vec![signer.clone()]),
            broadcast_only: false,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_payer: signer,
        }
    }

    pub fn default_from_enstablished(signers: BTreeSet<Alias>, gas_payer: Alias) -> Self {
        Self {
            signers,
            broadcast_only: false,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_payer,
        }
    }

    pub fn from_signers(signers: BTreeSet<Alias>) -> Self {
        TxSettings {
            signers: signers.clone(),
            broadcast_only: false,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_payer: signers.first().unwrap().to_owned(),
        }
    }
}
