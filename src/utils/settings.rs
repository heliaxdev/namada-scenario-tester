use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    #[serde(rename = "broadcast-only")]
    pub broadcast_only: Option<bool>
}
