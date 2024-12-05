use chrono::{DateTime, Utc};
use serde_json;
use serde::{self, Serialize, Deserialize};

use super::BlueskyApiProfileViewBasic;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineReasonRepost {
	pub by: BlueskyApiProfileViewBasic,
	pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiTimelineReason {
	#[serde(rename = "app.bsky.feed.defs#reasonRepost")]
	Repost(BlueskyApiTimelineReasonRepost),
	#[serde(rename = "app.bsky.feed.defs#reasonPin")]
	Pin,
}