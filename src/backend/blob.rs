use std::{fs::File, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::defs;

use super::{BlueskyApiError, ClientBackend};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub blob: defs::Blob,
}

impl ClientBackend {
    pub async fn upload_blob(&mut self, buffer: Vec<u8>) -> Result<defs::Blob, BlueskyApiError> {
        let req = self.client.post(format!("{}/xrpc/com.atproto.repo.uploadBlob", self.user_pds)).body(buffer).header("content-type", "image/*");
        let req = self.make_request(req).await?;

        let res: Result<Response, serde_json::Error> = serde_json::from_str(&req);
        if let Err(err) = res {
            return Err(BlueskyApiError::ParseError(err, req));
        }

        Ok(res.unwrap().blob)
    }
}