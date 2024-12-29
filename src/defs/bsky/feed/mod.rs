use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
use serde_json;
use std::sync::Arc;

pub mod defs;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Post {
	pub text: String,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<crate::defs::bsky::richtext::Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Arc<crate::defs::bsky::embed::Variant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}