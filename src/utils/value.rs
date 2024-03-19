use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "ref")]
    Ref { value: u64, field: String },
    #[serde(rename = "value")]
    Value { value: String },
    #[serde(rename = "fuzz")]
    Fuzz {},
}

impl Value {
    pub fn r(value: u64, field: String) -> Self {
        Self::Ref { value, field }
    }

    pub fn v(value: String) -> Self {
        Self::Value { value }
    }

    pub fn f() -> Self {
        Self::Fuzz {}
    }
}
