use super::{responses, BlueskyApiError, ClientBackend};

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

        if let Ok(fin) = fin {
            /*
            for mut obj in &fin.feed {
                if let Some(guh) = &obj.post.record.facets {
                    let mut guh_clone = guh.clone();
                    guh_clone.sort_by(|a, b| {
                        a.index.byte_start.cmp(&b.index.byte_start)
                    });
                    obj.post.record.facets = Some(guh_clone);
                }
            }
            */
            return Ok(fin);
        } else {
            return Err(BlueskyApiError::ParseError(format!("{:?}", fin.err().unwrap())));
        }
    }
}
