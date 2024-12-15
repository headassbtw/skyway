use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};

use crate::defs::bsky::actor::defs::ProfileViewBasic;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: crate::defs::bsky::actor::defs::ProfileViewBasic,
    pub record: crate::backend::responses::timeline::BlueskyApiTimelinePostRecord,
    pub indexed_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<crate::defs::bsky::embed::Variant>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<crate::defs::bsky::feed::defs::ViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerState {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedCursorPair {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub feed: Vec<FeedViewPost>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPost {
    pub post: Arc<Mutex<PostView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub root: RelatedPostVariant,
    pub parent: RelatedPostVariant,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grandparent_author: Option<ProfileViewBasic>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Reason {
    #[serde(rename = "app.bsky.feed.defs#reasonRepost")]
    Repost(ReasonRepost),
    #[serde(rename = "app.bsky.feed.defs#reasonPin")]
    Pin,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasonRepost {
    pub by: ProfileViewBasic,
    pub indexed_at: DateTime<Utc>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum RelatedPostVariant {
    #[serde(rename = "app.bsky.feed.defs#postView")]
    Post(PostView),
    #[serde(rename = "app.bsky.feed.defs#notFoundPost")]
    NotFound(NotFoundPost),
    #[serde(rename = "app.bsky.feed.defs#blockedPost")]
    Blocked(BlockedPost),
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum ThreadPostVariant {
    #[serde(rename = "app.bsky.feed.defs#threadViewPost")]
    ThreadView(ThreadViewPost),
    #[serde(rename = "app.bsky.feed.defs#notFoundPost")]
    NotFound(NotFoundPost),
    #[serde(rename = "app.bsky.feed.defs#blockedPost")]
    Blocked(BlockedPost),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadViewPost {
    pub post: Arc<Mutex<PostView>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Arc<ThreadPostVariant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<ThreadPostVariant>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotFoundPost {
    pub uri: String,
    pub not_found: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedPost {
    pub uri: String,
    pub blocked: bool,
    pub author: BlockedAuthor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedAuthor {
    pub did: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
}
