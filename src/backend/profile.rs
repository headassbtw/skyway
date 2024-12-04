use super::{BlueskyApiError, ClientBackend};
use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};
//use serde_json;

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlueskyApiProfileAssociatedChat {
  All,
  None,
  Following
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiProfileAssociated {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lists: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub feed_gens: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub starter_packs: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub labeler: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub chat: Option<BlueskyApiProfileAssociatedChat>
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiProfileViewer {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub muted: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub muted_by_list: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub blocked_by: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub blocking: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub blocking_by_list: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub following: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub followed_by: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub known_followers: Option<Vec<serde_json::Value>>
}


#[derive(std::fmt::Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiProfile {
	pub did: String,
	pub handle: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub display_name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub banner: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub followers_count: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follows_count: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub posts_count: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub associated: Option<BlueskyApiProfileAssociated>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub joined_via_starter_pack: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub indexed_at: Option<DateTime<Utc>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub created_at: Option<DateTime<Utc>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub viewer: Option<BlueskyApiProfileViewer>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub labels: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pinned_post: Option<serde_json::Value>,

}

impl ClientBackend {
	pub async fn get_profile_self(&mut self) -> Result<BlueskyApiProfile, BlueskyApiError> {
		self.get_profile(self.did.clone()).await
	}

	pub async fn get_profile(&mut self, did: String) -> Result<BlueskyApiProfile, BlueskyApiError> {
		let request = self.client.get(format!("{}/xrpc/app.bsky.actor.getProfile?actor={}", self.standard_pds, did));
		let req = self.make_request(request).await;
		if let Err(err) = req { return Err(err); }
		let req = req.unwrap();

		let parse: Result<BlueskyApiProfile, serde_json::Error> = serde_json::from_str(&req);
		if let Err(err) = parse {
			return Err(BlueskyApiError::ParseError(format!("{}", err)));
		} let parse = parse.unwrap();

		return Ok(parse);
	}
}