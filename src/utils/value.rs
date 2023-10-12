use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "ref")]
    Ref { value: u64 },
    #[serde(rename = "value")]
    Value { value: String },
    #[serde(rename = "fuzz")]
    Fuzz {},
}
