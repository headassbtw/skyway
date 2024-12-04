use serde_json;
use serde::{self, Serialize, Deserialize};

use super::BlueskyApiTimelinePostResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiTimelineReasonReply {
	#[serde(rename = "app.bsky.feed.defs#postView")]
	Post(BlueskyApiTimelinePostResponse),
	#[serde(rename = "app.bsky.feed.defs#notFoundPost")]
	NotFound,
	#[serde(rename = "app.bsky.feed.defs#blockedPost")]
	Blocked,

}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineReasonReplyChunk {
	pub root: BlueskyApiTimelineReasonReply,
	pub parent: BlueskyApiTimelineReasonReply,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub grandparent_author: Option<serde_json::Value>,
}