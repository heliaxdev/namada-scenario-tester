use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "ref")]
    Ref { value: u64, field: String },
    #[serde(rename = "value")]
    Value { value: String },
    #[serde(rename = "fuzz")]
    Fuzz { value: Option<u64> },
}

impl Value {
    pub fn r(value: u64, field: String) -> Self {
        Self::Ref { value, field }
    }

    pub fn v(value: String) -> Self {
        Self::Value { value }
    }

    pub fn f(value: Option<u64>) -> Self {
        Self::Fuzz { value }
    }
}
