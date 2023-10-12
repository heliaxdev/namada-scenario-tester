use serde::Deserialize;

use crate::utils::value::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct TxTransferParametersDto {
    source: Value,
    target: Value,
    amount: Value,
    token: Value,
}
