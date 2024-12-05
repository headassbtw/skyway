use reason::BlueskyApiTimelineReason;
use reply::BlueskyApiTimelineReasonReplyChunk;
use serde_json;
use serde::{self, Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::backend::record::BlueskyApiReplyRef;

pub mod embed;
pub mod reason;
pub mod reply;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineResponse {
	pub feed: Vec<BlueskyApiTimelineResponseObject>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineResponseRaw {
	pub feed: Vec<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineResponseObject {
	pub post: BlueskyApiPostView,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reply: Option<BlueskyApiTimelineReasonReplyChunk>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reason: Option<BlueskyApiTimelineReason>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub feed_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelinePostAuthor {
	pub did: String,
	pub handle: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub display_name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub associated: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub viewer: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub labels: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiRichTextIndex {
	pub byte_end: usize,
	pub byte_start: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiRichTextFeature {
	#[serde(rename = "app.bsky.richtext.facet#mention")]
	Mention(serde_json::Value),
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link(serde_json::Value),
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag(serde_json::Value)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiRichTextFacet {
	pub features: Vec<BlueskyApiRichTextFeature>,
	pub index: BlueskyApiRichTextIndex,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelinePostRecord {
	pub text: String,
	pub created_at: DateTime<Utc>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub facets: Option<Vec<BlueskyApiRichTextFacet>>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiFeedViewerState {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repost: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub like: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub thread_muted: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reply_disabled: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embedding_disabled: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pinned: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiPostView {
	pub uri: String,
	pub cid: String,
	pub author: BlueskyApiTimelinePostAuthor,
	pub record: BlueskyApiTimelinePostRecord,
	pub indexed_at: DateTime<Utc>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub embed: Option<embed::BlueskyApiTimelinePostEmbedView>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub reply_count: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repost_count: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub like_count: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub quote_count: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub viewer: Option<BlueskyApiFeedViewerState>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub labels: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub threadgate: Option<serde_json::Value>,
}