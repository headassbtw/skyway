use crate::defs::{self, bsky::{actor::defs::Preference, feed::defs::FeedCursorPair}};

use super::{BlueskyApiError, ClientBackend};

#[derive(Debug, serde::Deserialize)]
pub struct PreferencesResponse {
    pub preferences: Vec<Preference>
}

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
            return Err(BlueskyApiError::ParseError(err, req));
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
            return Err(BlueskyApiError::ParseError(err, req));
        }
        let mut res = parse.unwrap();
        for post in res.feed.iter_mut() {
            post.post = self.deduplicate_post(&mut post.post);
        };

        return Ok(res);
    }

    /// Get private preferences attached to the current account. Expected use is synchronization between multiple devices, and import/export during account migration. Requires auth.
    pub async fn get_preferences(&mut self) -> Result<Vec<Preference>, BlueskyApiError> {
        let request = self.client.get(format!("{}/xrpc/app.bsky.actor.getPreferences", self.user_pds));
        let req = self.make_request(request).await;
        if let Err(err) = req {
            return Err(err);
        }
        let req = req.unwrap();

        let parse: Result<PreferencesResponse, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = parse {
            return Err(BlueskyApiError::ParseError(err, req));
        }
        let parse = parse.unwrap();

        return Ok(parse.preferences);
    }
}
