use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};

use crate::backend::record::BlueskyApiRecord;

#[derive(Debug, Serialize, Deserialize)]
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