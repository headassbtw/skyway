use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
use serde_json;
use std::sync::Arc;

pub mod defs;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Post {
	pub text: String,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<crate::defs::bsky::richtext::Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Arc<crate::defs::bsky::embed::Variant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum ThreatGateAllow {
    /// Allow replies from actors mentioned in your post.
    #[serde(rename = "app.bsky.feed.threadgate#mentionRule")]
    Mention,
    /// Allow replies from actors who follow you.
    #[serde(rename = "app.bsky.feed.threadgate#followerRule")]
    Follower,
    /// Allow replies from actors you follow.
    #[serde(rename = "app.bsky.feed.threadgate#followingRule")]
    Following,
    /// Allow replies from actors on a list.
    #[serde(rename = "app.bsky.feed.threadgate#listRule")]
    List {
        list: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Record defining interaction gating rules for a thread (aka, reply controls). The record key (rkey) of the threadgate record must match the record key of the thread's root post, and that record must be in the same repository.
pub struct ThreadGate {
    /// Reference (AT-URI) to the post record.
    pub post: String,
    /// List of rules defining who can reply to this post. If value is an empty array, no one can reply. If value is undefined, anyone can reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<ThreatGateAllow>>,
    pub created_at: DateTime<Utc>,
    /// List of hidden reply URIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_replies: Option<Vec<String>>,
}