use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "ref")]
    Ref { value: u64, field: String },
    #[serde(rename = "value")]
    Value { value: String },
    #[serde(rename = "fuzz")]
    Fuzz { value: Option<u64> },
}
