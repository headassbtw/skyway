use std::borrow::BorrowMut;

use serde::{Deserialize, Serialize};

use crate::defs::bsky::feed::defs::ThreadPostVariant;

use super::{BlueskyApiError, ClientBackend};

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct BlueskyApiGetThreadResponse {
    pub thread: crate::defs::bsky::feed::defs::ThreadViewPost,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<serde_json::Value>,
}

impl ClientBackend {
    fn dedup_threadview(&mut self, var: &mut ThreadPostVariant) {
        match var {
            crate::defs::bsky::feed::defs::ThreadPostVariant::ThreadView(post) => {
                post.post = self.deduplicate_post(&mut post.post);
                
                if let Some(parent) = &post.parent {
                    self.dedup_threadview(&mut parent.lock().unwrap());
                }

                let replies = post.replies.borrow_mut();
                if let Some(replies) = replies {
                    for reply in replies.iter_mut() {
                        self.dedup_threadview(reply);
                    }
                }
            },
            _ => {}
        }
    }


    pub async fn get_thread(&mut self, uri: String, depth: Option<usize>, height: Option<usize>) -> Result<BlueskyApiGetThreadResponse, BlueskyApiError> {
        let depth = if let Some(depth) = depth {
            if depth > 1000 {
                1000
            } else {
                depth
            }
        } else {
            6
        };
        let height = if let Some(height) = height {
            if height > 1000 {
                1000
            } else {
                height
            }
        } else {
            80
        };

        let req = self.client.get(format!("{}/xrpc/app.bsky.feed.getPostThread?uri={uri}&depth={depth}&parentHeight={height}", self.user_pds));
        let req = self.make_request(req).await?;

        let parse: Result<BlueskyApiGetThreadResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(err, req));
        }
        let mut parse = parse.unwrap();

        // deduplication stuff
        parse.thread.post = self.deduplicate_post(&mut parse.thread.post);

        if let Some(replies) = parse.thread.replies.as_mut() {
            for reply in replies.iter_mut() {
                self.dedup_threadview(reply);
            }
        }

        return Ok(parse);
    }
}
