use std::sync::Arc;

use crate::defs::bsky::feed::StrongRef;

use super::{BlueskyApiError, ClientBackend};
use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiRecordPost {
    pub text: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<crate::defs::bsky::feed::ReplyRef>,
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
pub struct BlueskyApiRecordLike {
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiRecord {
    #[serde(rename = "app.bsky.feed.post")]
    Post(BlueskyApiRecordPost),
    #[serde(rename = "app.bsky.feed.like")]
    Like(BlueskyApiRecordLike),
    #[serde(rename = "app.bsky.feed.repost")]
    Repost(BlueskyApiRecordLike),
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiCreateRecordResponseCommit {
    pub cid: String,
    pub rev: String,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiCreateRecordResponse {
    pub uri: String,
    pub cid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<BlueskyApiCreateRecordResponseCommit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_status: Option<String>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiDeleteRecordResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<BlueskyApiCreateRecordResponseCommit>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CreateRecordRequest {
    pub repo: String,
    pub collection: String,
    pub record: BlueskyApiRecord,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeleteRecordRequest {
    pub repo: String,
    pub collection: String,
    pub rkey: String,
}

impl ClientBackend {
    pub async fn create_record(&mut self, record: BlueskyApiRecord) -> Result<BlueskyApiCreateRecordResponse, BlueskyApiError> {
        let nsid = match record {
            BlueskyApiRecord::Post(_) => "app.bsky.feed.post",
            BlueskyApiRecord::Like(_) => "app.bsky.feed.like",
            BlueskyApiRecord::Repost(_) => "app.bsky.feed.repost",
        };

        let contents = CreateRecordRequest { repo: self.did.clone(), collection: nsid.to_owned(), record };

        let body = serde_json::to_string::<CreateRecordRequest>(&contents);
        if let Err(err) = body {
            return Err(BlueskyApiError::ParseError(format!("{:?}", err)));
        }
        let body = body.unwrap();

        let req = self.client.post(format!("{}/xrpc/com.atproto.repo.createRecord", self.user_pds)).body(body).header("content-type", "application/json");
        let req = self.make_request(req).await?;

        let parse: Result<BlueskyApiCreateRecordResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(format!("Serialization Failed.\nJSON:{}\nError:{:?}", req, err)));
        }

        return Ok(parse.unwrap());
    }

    pub async fn delete_record(&mut self, rkey: String, nsid: String) -> Result<BlueskyApiDeleteRecordResponse, BlueskyApiError> {
        let contents = DeleteRecordRequest { repo: self.did.clone(), collection: nsid.to_owned(), rkey };

        let body = serde_json::to_string::<DeleteRecordRequest>(&contents);
        if let Err(err) = body {
            return Err(BlueskyApiError::ParseError(format!("{:?}", err)));
        }
        let body = body.unwrap();

        let req = self.client.post(format!("{}/xrpc/com.atproto.repo.deleteRecord", self.user_pds)).body(body).header("content-type", "application/json");
        let req = self.make_request(req).await?;

        let parse: Result<BlueskyApiDeleteRecordResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(format!("Serialization Failed.\nJSON:{}\nError:{:?}", req, err)));
        }

        return Ok(parse.unwrap());
    }
}
