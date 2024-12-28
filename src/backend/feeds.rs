use serde::Deserialize;

use super::{BlueskyApiError, ClientBackend};

use crate::defs::bsky::feed::defs::{GeneratorView, FeedCursorPair};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedGeneratorsResponse {
	pub feeds: Vec<GeneratorView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorFeedsResponse {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
	pub feeds: Vec<GeneratorView>,
}

impl ActorFeedsResponse {
	pub fn none() -> Self {
		ActorFeedsResponse {
    		cursor: None,
    		feeds: Vec::new(),
		}
	}
}


impl ClientBackend {
	/// Get a list of feeds (feed generator records) created by the actor (in the actor's repo).
	pub async fn get_actor_feeds(&mut self, did: String, cursor: Option<String>) -> Result<ActorFeedsResponse, BlueskyApiError> {
        let req = self.client.get(
        	format!("{}/xrpc/app.bsky.feed.getActorFeeds?actor={}{}", self.user_pds, did,
        		if let Some(cursor) = cursor { format!("&cursor={}", cursor) } else { String::new() })
        	);
        let req = self.make_request(req).await?;

        let res: Result<ActorFeedsResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = res {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        Ok(res.unwrap())
    }

    /// Get a hydrated feed from an actor's selected feed generator.
	pub async fn get_feed(&mut self, feed: String, cursor: Option<String>) -> Result<FeedCursorPair, BlueskyApiError> {
        let req = self.client.get(
        	format!("{}/xrpc/app.bsky.feed.getFeed?feed={}{}", self.user_pds, feed,
        		if let Some(cursor) = cursor { format!("&cursor={}", cursor) } else { String::new() })
        	);
        let req = self.make_request(req).await?;

        let res: Result<FeedCursorPair, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = res {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        Ok(res.unwrap())
    }

    /// Get information about a list of feed generators.
	pub async fn get_feed_generators(&mut self, feeds: Vec<String>) -> Result<Vec<GeneratorView>, BlueskyApiError> {
		let mut feeds_list = String::new();
		for feed in feeds {
			feeds_list.push_str(&format!("feeds={}&", feed));
		}
        let req = self.client.get(format!("{}/xrpc/app.bsky.feed.getFeedGenerators?{}", self.user_pds, feeds_list));
        let req = self.make_request(req).await?;

        let res: Result<FeedGeneratorsResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = res {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        Ok(res.unwrap().feeds)
    }
}