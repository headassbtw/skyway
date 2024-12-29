use crate::defs::bsky::richtext::{facet::Link, Facet, Feature, Index};

use super::{BlueskyApiError, ClientBackend};
use serde::{self, Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum BlueskyApiRecord {
    #[serde(rename = "app.bsky.feed.post")]
    Post(crate::defs::bsky::feed::Post),
    #[serde(rename = "app.bsky.feed.like")]
    Like(crate::defs::bsky::feed::Like),
    #[serde(rename = "app.bsky.feed.repost")]
    Repost(crate::defs::bsky::feed::Like),
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

fn link_detector(text: String) -> Vec<Facet> {
    let mut rtn = Vec::new();
    let mut start_idx: usize = 0;
    'find: loop {
        let detect =                       text[start_idx..].find("https://");
        let detect = if detect.is_none() { text[start_idx..].find("http://")  } else { detect };
        let detect = if detect.is_none() { text[start_idx..].find("steam://") } else { detect }; // lol
        if let Some(start) = detect {
            let start_absolute = start + start_idx;
            let end_absolute = if let Some(end) = text[start_absolute..].find([' ', ')', '\0','\n']) {
                start_absolute + end
            } else { text.len() };

            let facet = Facet {
                features: {
                    let mut rtn = Vec::new();
                    rtn.push(Feature::Link(Link {
                        uri: text[start_absolute..end_absolute].to_string(),
                    }));
                    rtn
                },
                index: Index {
                    byte_start: start_absolute,
                    byte_end: end_absolute,
                },
            };
            rtn.push(facet);
            start_idx = end_absolute;
        } else {
            break 'find;
        }
    }

    rtn
}

impl ClientBackend {
    pub async fn create_record(&mut self, record: BlueskyApiRecord) -> Result<BlueskyApiCreateRecordResponse, BlueskyApiError> {
        let (nsid, record) = match record {
            BlueskyApiRecord::Post(post) => {
                let post = if post.facets.is_some() {
                    post
                } else {
                    let mut post = post;
                    post.facets = Some(link_detector(post.text.clone()));
                    post
                };
                
                ("app.bsky.feed.post", BlueskyApiRecord::Post(post))
            },
            BlueskyApiRecord::Like(_) => ("app.bsky.feed.like", record),
            BlueskyApiRecord::Repost(_) => ("app.bsky.feed.repost", record),
        };

        let contents = CreateRecordRequest { repo: self.did.clone(), collection: nsid.to_owned(), record };

        let body = serde_json::to_string::<CreateRecordRequest>(&contents);
        if let Err(err) = body {
            return Err(BlueskyApiError::ParseError(err, String::new()));
        }
        let body = body.unwrap();

        let req = self.client.post(format!("{}/xrpc/com.atproto.repo.createRecord", self.user_pds)).body(body).header("content-type", "application/json");
        let req = self.make_request(req).await?;

        let parse: Result<BlueskyApiCreateRecordResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        return Ok(parse.unwrap());
    }

    pub async fn delete_record(&mut self, rkey: String, nsid: String) -> Result<BlueskyApiDeleteRecordResponse, BlueskyApiError> {
        let contents = DeleteRecordRequest { repo: self.did.clone(), collection: nsid.to_owned(), rkey };

        let body = serde_json::to_string::<DeleteRecordRequest>(&contents);
        if let Err(err) = body {
            return Err(BlueskyApiError::ParseError(err, String::new()));
        }
        let body = body.unwrap();

        let req = self.client.post(format!("{}/xrpc/com.atproto.repo.deleteRecord", self.user_pds)).body(body).header("content-type", "application/json");
        let req = self.make_request(req).await?;

        let parse: Result<BlueskyApiDeleteRecordResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        return Ok(parse.unwrap());
    }
}
