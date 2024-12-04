use serde_json;
use serde::{self, Serialize, Deserialize};

pub mod timeline;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueskyApiLoginResponse {
	pub access_jwt: String,
	pub refresh_jwt: String,
	pub handle: String,
	pub did: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub did_doc: Option<serde_json::Value>,
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