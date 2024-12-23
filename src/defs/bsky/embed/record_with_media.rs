use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum MediaVariant {
	#[serde(rename = "app.bsky.embed.images#view")]
	Images { images: Vec<crate::defs::bsky::embed::images::ViewImage> },
	#[serde(rename = "app.bsky.embed.video#view")]
	Video(crate::defs::bsky::embed::video::View),
	#[serde(rename = "app.bsky.embed.external#view")]
	External { external: crate::defs::bsky::embed::external::View },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// thanks.
pub struct RecordWrapper {
	pub record: crate::defs::bsky::embed::record::Variant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordWithMedia {
	pub record: RecordWrapper,
	pub media: MediaVariant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordWithMediaRaw(serde_json::Value);