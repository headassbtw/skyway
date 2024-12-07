use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewImage {
    pub thumb: String,
    pub fullsize: String,
    pub alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<super::AspectRatio>,
}
