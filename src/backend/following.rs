use std::sync::Arc;

use serde::Deserialize;
use crate::defs::bsky::actor::defs::ProfileView;
use super::{BlueskyApiError, ClientBackend};

#[derive(Deserialize)]
pub struct GetFollowersResponse {
	pub subject: Arc<ProfileView>,
	pub followers: Vec<Arc<ProfileView>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
}

impl ClientBackend {
	pub async fn get_followers(&mut self, did: String, cursor: String) -> Result<(String, Vec<Arc<ProfileView>>), BlueskyApiError> {
        let request = self.client.get(format!("{}/xrpc/app.bsky.graph.getFollowers?actor={}&cursor={}", self.user_pds, did, cursor));
        let req = self.make_request(request).await?;

        let parse: Result<GetFollowersResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(err, req));
        }
        //TODO: de-duplication
        let res = parse.unwrap();
        /*
        for post in res.followers.iter_mut() {
            
        };
        */

        return Ok((res.cursor.unwrap_or(String::new()), res.followers));
    }
}