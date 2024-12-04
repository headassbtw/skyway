use serde_json;
use serde::{self, Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineEmbedAspectRatio {
	pub width: u32,
	pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineEmbedImageView {
	pub thumb: String,
	pub fullsize: String,
	pub alt: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub aspect_ratio: Option<BlueskyApiTimelineEmbedAspectRatio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineEmbedExternalView {
	pub uri: String,
	pub title: String,
	pub description: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub thumb: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiTimelineEmbedVideoView {
	pub cid: String,
	pub playlist: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub thumbnail: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub alt: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub aspect_ratio: Option<BlueskyApiTimelineEmbedAspectRatio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiTimelineEmbedRecordView {
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
    PackView(serde_json::Value),

}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiTimelinePostEmbedView {
	#[serde(rename = "app.bsky.embed.images#view")]
	Images { images: Vec<BlueskyApiTimelineEmbedImageView> },
	#[serde(rename = "app.bsky.embed.video#view")]
	Video(BlueskyApiTimelineEmbedVideoView),
	#[serde(rename = "app.bsky.embed.external#view")]
	External { external: BlueskyApiTimelineEmbedExternalView },
	#[serde(rename = "app.bsky.embed.record#view")]
	Record { record: BlueskyApiTimelineEmbedRecordView },
	#[serde(rename = "app.bsky.embed.recordWithMedia#view")]
	RecordWithMedia(serde_json::Value),
}