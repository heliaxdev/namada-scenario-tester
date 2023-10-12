use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TxParametersDto {
    outcome: String,
    id: String,
}
