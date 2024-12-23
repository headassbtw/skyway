use serde::{self, Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::defs::bsky::graph::defs::{ListView, StarterPackViewBasic};
use crate::backend::record::BlueskyApiRecord;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Variant {
    #[serde(rename = "app.bsky.embed.record#viewRecord")]
    Record(Record),
    #[serde(rename = "app.bsky.embed.record#viewNotFound")]
    NotFound(NotFound),
    #[serde(rename = "app.bsky.embed.record#viewBlocked")]
    Blocked(Blocked),
    #[serde(rename = "app.bsky.embed.record#viewDetached")]
    Detached(Detached),
    #[serde(rename = "app.bsky.feed.defs#generatorView")]
    FeedGenerator(serde_json::Value),
    #[serde(rename = "app.bsky.graph.defs#listView")]
    List(ListView),
    #[serde(rename = "app.bsky.labeler.defs#labelerView")]
    Labeler(serde_json::Value),
    #[serde(rename = "app.bsky.graph.defs#starterPackViewBasic")]
    PackView(StarterPackViewBasic),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub uri: String,
    pub cid: String,
    pub author: crate::defs::bsky::actor::defs::ProfileViewBasic,
    pub value: BlueskyApiRecord,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<serde_json::Value>,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotFound {
    pub uri: String,
    pub not_found: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blocked {
    pub uri: String,
    pub blocked: bool,
    pub author: crate::defs::bsky::feed::defs::BlockedAuthor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Detached {
    pub uri: String,
    pub detached: bool,
}