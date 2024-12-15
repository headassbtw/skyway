use crate::defs::{self, bsky::feed::defs::{FeedCursorPair, FeedViewPost}};

use super::{BlueskyApiError, ClientBackend};

impl ClientBackend {
    pub async fn get_profile_self(&mut self) -> Result<defs::bsky::actor::defs::ProfileViewDetailed, BlueskyApiError> {
        self.get_profile(self.did.clone()).await
    }

    pub async fn get_profile(&mut self, did: String) -> Result<defs::bsky::actor::defs::ProfileViewDetailed, BlueskyApiError> {
        let request = self.client.get(format!("{}/xrpc/app.bsky.actor.getProfile?actor={}", self.user_pds, did));
        let req = self.make_request(request).await;
        if let Err(err) = req {
            return Err(err);
        }
        let req = req.unwrap();

        let parse: Result<defs::bsky::actor::defs::ProfileViewDetailed, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(format!("{}", err)));
        }
        let parse = parse.unwrap();

        return Ok(parse);
    }

    pub async fn get_author_feed(&mut self, did: String, cursor: String) -> Result<FeedCursorPair, BlueskyApiError> {
        let request = self.client.get(format!("{}/xrpc/app.bsky.feed.getAuthorFeed?actor={}&cursor={}", self.user_pds, did, cursor));
        let req = self.make_request(request).await;
        if let Err(err) = req {
            return Err(err);
        }
        let req = req.unwrap();

        let parse: Result<FeedCursorPair, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(format!("{}", err)));
        }
        let parse = parse.unwrap();

        return Ok(parse);
    }
}
