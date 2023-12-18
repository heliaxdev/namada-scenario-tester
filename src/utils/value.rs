use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
#[serde(tag = "type")]
pub enum Value {
    /// Reference to a value stored from a task
    #[serde(rename = "ref")]
    Ref {
        /// Step ID
        value: u64,
        /// Field name (e.g. "alias", "public-key", "state")
        field: String,
    },
    /// Literal value
    #[serde(rename = "value")]
    Value { value: String },
    /// A value at a randomized index
    #[serde(rename = "fuzz")]
    Fuzz { value: Option<u64> },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Ref { value, field } => write!(f, "Ref {value} - {field}"),
            Value::Value { value } => write!(f, "Value {value}"),
            Value::Fuzz { value } => write!(f, "Fuzz {value:?}"),
        }
    }
}
