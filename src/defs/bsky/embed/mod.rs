use serde::{self, Deserialize, Serialize};

pub mod external;
pub mod images;
pub mod record;
pub mod record_with_media;
pub mod video;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Variant {
    #[serde(rename = "app.bsky.embed.images#view")]
    Images { images: Vec<images::ViewImage> },
    #[serde(rename = "app.bsky.embed.video#view")]
    Video(video::View),
    #[serde(rename = "app.bsky.embed.external#view")]
    External { external: external::View },
    #[serde(rename = "app.bsky.embed.record#view")]
    Record { record: record::Variant },
    #[serde(rename = "app.bsky.embed.recordWithMedia#view")]
    RecordWithMedia(record_with_media::RecordWithMedia),

    #[serde(rename = "app.bsky.embed.images")]
    ImagesRaw { images: Vec<images::Image> },
    #[serde(rename = "app.bsky.embed.video")]
    VideoRaw(serde_json::Value),
    #[serde(rename = "app.bsky.embed.external")]
    ExternalRaw(serde_json::Value),
    #[serde(rename = "app.bsky.embed.record")]
    RecordRaw(serde_json::Value),
    #[serde(rename = "app.bsky.embed.recordWithMedia")]
    RecordWithMediaRaw(record_with_media::RecordWithMediaRaw),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}
