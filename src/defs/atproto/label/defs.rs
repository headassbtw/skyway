use serde::{Deserialize, Serialize};

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label (serde_json::Value);