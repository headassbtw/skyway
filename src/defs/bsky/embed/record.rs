use serde::{self, Deserialize, Serialize};

use crate::defs::bsky::graph::defs::StarterPackViewBasic;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum View {
    #[serde(rename = "app.bsky.embed.record#viewRecord")]
    Record(serde_json::Value),
    #[serde(rename = "app.bsky.embed.record#viewNotFound")]
    NotFound(serde_json::Value),
    #[serde(rename = "app.bsky.embed.record#viewBlocked")]
    Blocked(serde_json::Value),
    #[serde(rename = "app.bsky.embed.record#viewDetached")]
    Detached(serde_json::Value),
    #[serde(rename = "app.bsky.feed.defs#generatorView")]
    FeedGenerator(serde_json::Value),
    #[serde(rename = "app.bsky.graph.defs#listView")]
    List(serde_json::Value),
    #[serde(rename = "app.bsky.labeler.defs#labelerView")]
    Labeler(serde_json::Value),
    #[serde(rename = "app.bsky.graph.defs#starterPackViewBasic")]
    PackView(StarterPackViewBasic),
}
