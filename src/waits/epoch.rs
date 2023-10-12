use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct EpochParametersDto {
    pub from: Option<String>,
    pub r#for: Option<u64>,
    pub to: Option<u64>,
}
