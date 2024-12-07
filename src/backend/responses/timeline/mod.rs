use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
use serde_json;

use crate::backend::record::BlueskyApiReplyRef;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineResponse {
    pub feed: Vec<crate::defs::bsky::feed::defs::FeedViewPost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelinePostRecord {
    pub text: String,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<crate::defs::bsky::richtext::Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<BlueskyApiReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<serde_json::Value>>,
}
