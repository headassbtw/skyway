use serde::{self, Deserialize, Serialize};

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct ListViewBasic(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct StarterPackViewBasic(serde_json::Value);
