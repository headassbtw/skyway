use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;

use super::{
    main::{BlueskyLoginResponse, BlueskyLoginResponseError, LoginInformation},
    ClientBackend,
};
use base64::prelude::*;

#[allow(dead_code)]
#[derive(Deserialize)]
struct JwtMidsection {
    scope: String,
    sub: String,
    iat: usize,
    exp: usize,
    aud: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtProtoService {
    pub id: String,
    pub r#type: String,
    pub service_endpoint: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// THERE'S NO SPEC FOR THIS. IT DOESN'T FUCKING EXIST. I'm guessing this on API responses and github issues.
pub struct DidDoc {
    #[serde(rename = "@context")]
    pub context: Vec<serde_json::Value>,
    pub id: String,
    pub also_known_as: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<serde_json::Value>>,
    pub service: Vec<AtProtoService>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiLoginResponse {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_doc: Option<DidDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_confirmed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_auth_factor: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl ClientBackend {
    pub fn new() -> Self {
        Self {
            did: String::new(),
            standard_pds: "https://public.api.bsky.app".into(),
            user_pds: "https://bsky.social".into(),
            access_token: String::new(),
            refresh_token: String::new(),
            access_token_expiry: Utc::now(),
            client: reqwest::Client::new(),
            post_cache: HashMap::new(),
        }
    }

    async fn handle_login_response(&mut self, req: RequestBuilder) -> BlueskyLoginResponse {
        let res = req.send().await;

        if res.is_err() {
            return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Network(format!("{:?}", res)));
        }
        let res = res.unwrap();

        match res.status() {
            StatusCode::UNAUTHORIZED => return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Unauthorized),
            StatusCode::BAD_REQUEST => return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Network("Bad Request".into())),
            StatusCode::TOO_MANY_REQUESTS => return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Network("Rate Limited".into())),
            StatusCode::OK => {
                let jason_bytes = if let Ok(res) = res.bytes().await { res } else { return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Generic("Failed to read response".into())) };
                let jason: &str = if let Ok(res) = std::str::from_utf8(&jason_bytes) { res } else { return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Generic("Failed to decode response".into())) };
                let response: Result<BlueskyApiLoginResponse, serde_json::Error> = serde_json::from_str(jason);
                if response.is_err() {
                    return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Generic(format!("{:?}\n{}", response, jason)));
                }
                let response = response.unwrap();

                self.did = response.did;
                self.access_token = response.access_jwt;
                if let Some(did_doc) = response.did_doc {
                    if did_doc.service.len() > 0 {
                        self.user_pds = did_doc.service[0].service_endpoint.clone();
                    }
                }

                if response.active.is_none() || !response.active.unwrap() {
                    let reason = response.status;
                    if reason == Some("takendown".to_owned()) {
                        return BlueskyLoginResponse::Error(BlueskyLoginResponseError::AccountTakenDown);
                    } else if reason == Some("suspended".to_owned()) {
                        return BlueskyLoginResponse::Error(BlueskyLoginResponseError::AccountSuspended);
                    } else if reason == Some("deactivated".to_owned()) {
                        return BlueskyLoginResponse::Error(BlueskyLoginResponseError::AccountDeactivated);
                    } else {
                        return BlueskyLoginResponse::Error(BlueskyLoginResponseError::AccountInactive);
                    }
                }

                let token_list: Vec<&str> = self.access_token.split(".").collect();
                let mut proc = String::new();
                proc.push_str(token_list[1]);
                for _ in 0..(token_list[1].len() % 4) {
                    proc.push('=');
                }
                let decode = BASE64_STANDARD.decode(proc);

                let expiry: Option<DateTime<Utc>> = if let Ok(dec) = decode {
                    if let Ok(payload) = String::from_utf8(dec) {
                        let jason: Result<JwtMidsection, serde_json::Error> = serde_json::from_str(&payload);
                        if let Ok(jwt) = jason {
                            Some(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(jwt.exp as i64, 0 as u32), Utc))
                        } else {
                            println!("couldn't parse jwt json");
                            None
                        }
                    } else {
                        println!("couldn't parse jwt string");
                        None
                    }
                } else {
                    println!("couldn't base64 decode {:?}", decode);
                    None
                };

                if let Some(expiry) = expiry {
                    self.access_token_expiry = expiry;
                } else {
                    // expiry is usually two hours, so we'll fall back to that, but setting it to the proper one is cool
                    self.access_token_expiry = Utc::now().checked_add_signed(TimeDelta::hours(2)).unwrap_or(Utc::now());
                }

                self.refresh_token = response.refresh_jwt.clone();
                
                return BlueskyLoginResponse::Success(LoginInformation {
                    did: self.did.clone(),
                    refresh_token: response.refresh_jwt
                });
            }
            _ => {
                println!("Generic error: {:?}", res);
                return BlueskyLoginResponse::Error(BlueskyLoginResponseError::Generic(format!("{:?}", res)));
            }
        }
    }

    /// Fresh login
    pub async fn login(&mut self, handle: String, password: String) -> BlueskyLoginResponse {
        let mut map = HashMap::new();
        map.insert("identifier", handle);
        map.insert("password", password);

        let req = self.client.post(format!("{}/xrpc/com.atproto.server.createSession", self.user_pds)).json(&map);

        //if self.profile.is_none() {

        self.handle_login_response(req).await
    }

    /*
    /// Fresh login, the user has 2FA
    pub async fn login_2fa(&mut self, handle: String, password: String, two_factor_code: String) -> BlueskyLoginResponse {
        todo!()
    }
    */

    /// Login from a cached token
    pub async fn login_refresh(&mut self, refresh_token: String) -> BlueskyLoginResponse {
        println!("Refreshing login");
        let req = self.client.post(format!("{}/xrpc/com.atproto.server.refreshSession", self.user_pds)).bearer_auth(refresh_token);

        self.handle_login_response(req).await
    }
}
