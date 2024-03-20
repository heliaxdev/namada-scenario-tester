use std::fmt::Display;

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
}

impl Account {
    pub fn new(
        alias: Alias,
        implicit_addresses: Vec<Alias>,
        address_type: AddressType,
        threshold: u64,
    ) -> Self {
        Self {
            threshold,
            implicit_addresses,
            address_type,
            alias,
        }
    }

    pub fn new_implicit_address(alias: Alias) -> Self {
        Self::new(alias.clone(), vec![alias], AddressType::Implicit, 1)
    }

    pub fn new_enstablished_address(alias: Alias, pks: Vec<Alias>, threshold: u64) -> Self {
        Self::new(alias.clone(), pks, AddressType::Enstablished, threshold)
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
