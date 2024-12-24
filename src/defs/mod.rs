use serde::{Deserialize, Serialize};

pub mod atproto;
pub mod bsky;
pub mod chat;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlobRef {
    #[serde(rename = "$link")]
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    /// ALWAYS "blob". probably.
    #[serde(rename = "$type")]
    pub r#type: String,
    pub mime_type: String,
    pub r#ref: BlobRef,
    pub size: usize,
}