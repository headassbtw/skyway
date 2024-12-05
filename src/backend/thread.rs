use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};

use super::{responses::timeline::BlueskyApiPostView, BlueskyApiError, ClientBackend};

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
pub struct BlueskyApiThreadViewPost {
	pub post: Arc<Mutex<BlueskyApiPostView>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent: Option<Arc<BlueskyApiThreadResponse>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub replies: Option<Vec<BlueskyApiThreadResponse>>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "$type")]
pub enum BlueskyApiThreadResponse {
	#[serde(rename = "app.bsky.feed.defs#threadViewPost")]
	ThreadView(BlueskyApiThreadViewPost),
	#[serde(rename = "app.bsky.feed.defs#notFoundPost")]
	NotFound(serde_json::Value),
	#[serde(rename = "app.bsky.feed.defs#blockedPost")]
	Blocked(serde_json::Value)
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
pub struct BlueskyApiGetThreadResponse {
	pub thread: BlueskyApiThreadResponse,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub threadgate: Option<serde_json::Value>,
}

#[derive(std::fmt::Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlueskyApiGetThreadRequest {
	pub uri: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub depth: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent_height: Option<usize>,
}

impl ClientBackend {
	pub async fn get_thread(&mut self, uri: String, depth: Option<usize>, height: Option<usize>) -> Result<BlueskyApiGetThreadResponse, BlueskyApiError> {
		let depth = 
			if let Some(depth) = depth {
				if depth > 1000 { 1000 } else { depth }
			} else { 6 };
		let height =
			if let Some(height) = height {
				if height > 1000 { 1000 } else { height }
			} else { 80 };

		let req = self.client.get(format!("{}/xrpc/app.bsky.feed.getPostThread?uri={uri}&depth={depth}&parentHeight={height}", self.user_pds));
		let req = self.make_request(req).await?;

		let parse: Result<BlueskyApiGetThreadResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(format!("Serialization Failed.\nJSON:{}\nError:{:?}", req, err)));
        }

        return Ok(parse.unwrap());
	}
}