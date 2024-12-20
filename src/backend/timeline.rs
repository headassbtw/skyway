use std::borrow::BorrowMut;

use super::{BlueskyApiError, ClientBackend};

use crate::defs::bsky::feed::defs::FeedCursorPair;

impl ClientBackend {
    pub async fn get_timeline(&mut self, cursor: Option<String>, limit: Option<u32>) -> Result<FeedCursorPair, BlueskyApiError> {
        let limit = if let Some(limit) = limit { limit } else { 50 };
        let cursor = if let Some(cursor) = cursor { format!("&cursor={}", cursor) } else { String::new() };
        let target = format!("{}/xrpc/app.bsky.feed.getTimeline?limit={}{}", self.user_pds, limit, cursor);

        let request = self.make_request(self.client.get(target)).await;

        if let Err(err) = request {
            return Err(err);
        }
        let text = request.unwrap();

        let fin: Result<FeedCursorPair, serde_json::Error> = serde_json::from_str(&text);

        match fin {
            Ok(fin) => {
                // /let rtn = FeedCursorPair { cursor: fin.cursor, feed: Vec::new() };
                for post in &fin.feed {
                    let mut post = post.post.lock().unwrap();
                    let post = post.borrow_mut();
                    if let Some(fuck) = post.record.facets.as_mut() {
                        fuck.sort_by(|a,b| { a.index.byte_start.cmp(&b.index.byte_start) });
                    }
                };
                return Ok(fin)
            },
            Err(err) => return Err(BlueskyApiError::ParseError(err, text)),
        }
    }
}
