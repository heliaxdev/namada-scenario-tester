use serde::{Deserialize, Serialize};

use super::value::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxSettings {
    #[serde(rename = "broadcast-only")]
    pub broadcast_only: Option<bool>,
    #[serde(rename = "gas-token")]
    pub gas_token: Option<Value>,
}
