use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalMessage {
    pub mint: String,
    pub is_suspicious: bool,
    pub msg: String,
}

impl SignalMessage {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
