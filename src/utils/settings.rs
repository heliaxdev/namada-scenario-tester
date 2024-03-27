use serde::{Deserialize, Serialize};

use crate::entity::address::AccountIndentifier;

use super::value::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxSettingsDto {
    #[serde(rename = "broadcast-only")]
    pub broadcast_only: Option<bool>,
    #[serde(rename = "gas-token")]
    pub gas_token: Option<Value>,
    #[serde(rename = "gas-payer")]
    pub gas_payer: Option<Value>,
    #[serde(rename = "signers")]
    pub signers: Option<Vec<Value>>,
    #[serde(rename = "expiration")]
    pub expiration: Option<Value>,
    #[serde(rename = "gas-limit")]
    pub gas_limit: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct TxSettings {
    pub broadcast_only: bool,
    pub gas_token: Option<AccountIndentifier>,
    pub gas_payer: Option<AccountIndentifier>,
    pub signers: Option<Vec<AccountIndentifier>>,
    pub expiration: Option<u64>,
    pub gas_limit: Option<u64>,
}

impl TxSettingsDto {
    pub fn from_dto(&self) -> TxSettings {
        let broadcast_only = self.broadcast_only.unwrap_or(false);
        let gas_token = match self.gas_token.clone() {
            Some(Value::Value { value }) => Some(AccountIndentifier::Alias(value)),
            _ => None,
        };
        let gas_payer = match self.gas_payer.clone() {
            Some(Value::Value { value }) => Some(AccountIndentifier::Alias(value)),
            _ => None,
        };
        let signers = self.signers.clone().map(|signers| {
            signers
                .into_iter()
                .filter_map(|signer| match signer {
                    Value::Value { value } => Some(AccountIndentifier::Alias(value)),
                    _ => None,
                })
                .collect::<Vec<AccountIndentifier>>()
        });
        let expiration = match self.expiration.clone() {
            Some(Value::Value { value }) => Some(value.parse::<u64>().unwrap()),
            _ => None,
        };
        let gas_limit = match self.gas_limit.clone() {
            Some(Value::Value { value }) => Some(value.parse::<u64>().unwrap()),
            _ => None,
        };

        TxSettings {
            broadcast_only,
            gas_token,
            gas_payer,
            signers,
            expiration,
            gas_limit,
        }
    }
}
