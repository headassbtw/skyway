use facet::{Link, Mention, Tag};
use serde::{self, Deserialize, Serialize};

pub mod facet;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Facet {
    pub features: Vec<Feature>,
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub byte_end: usize,
    pub byte_start: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Feature {
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention(Mention),
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link(Link),
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag(Tag),
}
