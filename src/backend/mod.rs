use chrono::{DateTime, Utc};
use reqwest::{RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

pub mod login;
pub mod main;
pub mod profile;
pub mod record;
pub mod responses;
pub mod simple_actions;
pub mod thread;
pub mod timeline;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlueskyApiErrorMessage {
    pub error: String,
    pub message: String,
}

#[derive(Debug)]
pub enum BlueskyApiError {
    BadRequest(BlueskyApiErrorMessage),
    Unauthorized(BlueskyApiErrorMessage),
    NetworkError(String),
    ParseError(String),
}

pub struct ClientBackend {
    pub(self) did: String,
    /// Used for general unauthenticated things, lookups, profiles, most things
    pub standard_pds: String,
    /// Used for skeeting, and any authenticated actions
    user_pds: String,
    access_token: String,
    refresh_token: String,
    pub access_token_expiry: DateTime<Utc>,
    client: reqwest::Client,
}

impl ClientBackend {
    pub async fn make_request(&mut self, request: RequestBuilder) -> Result<String, BlueskyApiError> {
        if self.access_token_expiry < Utc::now() {
            println!("Token was outdated, refreshing...");
            self.login_refresh(self.refresh_token.clone()).await;
            println!("Refreshed.");
        }

        let request = request.bearer_auth(&self.access_token);
        println!("{:?}", request);
        let response = request.send().await;

        if let Err(err) = response {
            return Err(BlueskyApiError::NetworkError(format!("{:?}", err)));
        }
        let response = response.unwrap();

        let status_code = response.status();

        let string = response.text().await;
        if let Err(err) = string {
            return Err(BlueskyApiError::ParseError(format!("{:?}", err)));
        }
        let string = string.unwrap();

        if status_code == StatusCode::UNAUTHORIZED {
            let error: Result<BlueskyApiErrorMessage, serde_json::Error> = serde_json::from_str(&string);
            if let Err(err) = error {
                return Err(BlueskyApiError::ParseError(format!("{:?}", err)));
            }
            return Err(BlueskyApiError::Unauthorized(error.unwrap()));
        }

        if status_code == StatusCode::BAD_REQUEST {
            let error: Result<BlueskyApiErrorMessage, serde_json::Error> = serde_json::from_str(&string);
            if let Err(err) = error {
                return Err(BlueskyApiError::ParseError(format!("{:?}", err)));
            }
            return Err(BlueskyApiError::BadRequest(error.unwrap()));
        }

        return Ok(string);
    }
}
