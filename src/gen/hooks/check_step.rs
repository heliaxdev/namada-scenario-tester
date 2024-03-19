use std::fmt::Display;

use derive_builder::Builder;

use crate::step::Hook;

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct CheckStep {
    inner: u64,
}

impl CheckStep {
    pub fn new(step: u64) -> Self {
        Self { inner: step }
    }
}

impl Hook for CheckStep {
    fn to_json(&self) -> String {
        todo!()
    }
}

impl Display for CheckStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check step {}", self.inner)
    }
}
