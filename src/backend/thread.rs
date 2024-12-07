use serde::{Deserialize, Serialize};

use super::{BlueskyApiError, ClientBackend};

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct BlueskyApiGetThreadResponse {
    pub thread: crate::defs::bsky::feed::defs::ThreadViewPost,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<serde_json::Value>,
}

impl ClientBackend {
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
            return Err(BlueskyApiError::ParseError(format!("Serialization Failed.\nJSON:{}\nError:{:?}", req, err)));
        }

        return Ok(parse.unwrap());
    }
}
