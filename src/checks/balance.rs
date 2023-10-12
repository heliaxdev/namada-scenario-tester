use serde::Deserialize;

use crate::utils::value::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceParametersDto {
    amount: Value,
    address: Value,
}
