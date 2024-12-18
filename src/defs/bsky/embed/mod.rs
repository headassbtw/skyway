use serde::{self, Deserialize, Serialize};

pub mod external;
pub mod images;
pub mod record;
pub mod record_with_media;
pub mod video;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Variant {
    #[serde(rename = "app.bsky.embed.images#view")]
    Images { images: Vec<images::ViewImage> },
    #[serde(rename = "app.bsky.embed.images")]
    Images2(serde_json::Value),
    #[serde(rename = "app.bsky.embed.video#view")]
    Video(video::View),
    #[serde(rename = "app.bsky.embed.video")]
    Video2(serde_json::Value),
    #[serde(rename = "app.bsky.embed.external#view")]
    External { external: external::View },
    #[serde(rename = "app.bsky.embed.external")]
    External2(serde_json::Value),
    #[serde(rename = "app.bsky.embed.record#view")]
    Record { record: record::Variant },
    #[serde(rename = "app.bsky.embed.record")]
    Record2(serde_json::Value),
    #[serde(rename = "app.bsky.embed.recordWithMedia#view")]
    RecordWithMedia(serde_json::Value),
    #[serde(rename = "app.bsky.embed.recordWithMedia")]
    RecordWithMedia2(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}
