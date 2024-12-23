use serde::{self, Deserialize, Serialize};

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
pub struct ListViewBasic(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
pub struct ListView(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
pub struct StarterPackViewBasic(serde_json::Value);
