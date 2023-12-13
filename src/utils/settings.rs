use serde::Deserialize;

use super::value::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct TxSettings {
    #[serde(rename = "broadcast-only")]
    pub broadcast_only: Option<bool>,
    #[serde(rename = "gas-token")]
    pub gas_token: Option<Value>,
}
