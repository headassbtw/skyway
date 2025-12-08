use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
use crate::defs::bsky::actor::defs::ProfileViewBasic;
use crate::defs::bsky::feed::ThreatGateAllow;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: crate::defs::bsky::actor::defs::ProfileViewBasic,
    pub record: crate::defs::bsky::feed::Post,
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
    pub viewer: Option<ViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<ThreadGateView>,
}

impl PostView {
    pub fn url(&self) -> String {
        let id = self.uri.split("/").last().unwrap();
        let handle = if self.author.handle.eq("handle.invalid") { &self.author.did } else { &self.author.handle };
        format!("https://bsky.app/profile/{}/post/{}", handle, id)
    }

    pub fn can_reply(&self) -> bool {
        let disabled = if let Some(viewer) = &self.viewer {
            viewer.reply_disabled.unwrap_or(false)
        } else { false };

        if disabled { return false }

        let gated = if let Some(gate) = &self.threadgate {
            if let Some(gate) = &gate.record {
                if let Some(allow) = &gate.allow {
                    let mut met = false;
                    for condition in allow {
                        if met { return true }
                        // TODO(headassbtw): properly validate all of these
                        match condition {
                            ThreatGateAllow::Mention => {}
                            ThreatGateAllow::Follower => if let Some(viewer) = &self.author.viewer {
                                met = viewer.following.is_some()
                            }
                            ThreatGateAllow::Following => if let Some(viewer) = &self.author.viewer {
                                met = viewer.followed_by.is_some()
                            }
                            ThreatGateAllow::List { .. } => {}
                        }
                    }
                    return met;
                } else { false }
            } else { false }
        } else { false };

        !gated
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub parent: Option<Arc<Mutex<ThreadPostVariant>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<ThreadPostVariant>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotFoundPost {
    pub uri: String,
    pub not_found: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockedPost {
    pub uri: String,
    pub blocked: bool,
    pub author: BlockedAuthor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockedAuthor {
    pub did: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorView {
    pub uri: String,
    pub cid: String,
    pub did: String,
    pub creator: crate::defs::bsky::actor::defs::ProfileView,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_facets: Option<Vec<crate::defs::bsky::richtext::Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepts_interactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<crate::defs::atproto::label::defs::Label>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<GeneratorViewerState>,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadGateView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record: Option<crate::defs::bsky::feed::ThreadGate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<crate::defs::bsky::graph::defs::ListViewBasic>>,
}