use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};

use crate::defs;

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewBasic {
    pub did: String,
    pub handle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<ProfileAssociated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl ProfileViewBasic {
    /// For easily getting a string that always and considely identifies the user.
    // Shorthand for checking if the display name is empty, returning it if not, and returning the handle if so.
    pub fn easy_name(&self) -> &str {
        if let Some(dn) = &self.display_name {
            if dn.len() > 0 {
                return dn;
            } else {
                return &self.handle;
            }
        } else {
            return &self.handle;
        }
    }
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewDetailed {
    pub did: String,
    pub handle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follows_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posts_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<ProfileAssociated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_via_starter_pack: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_post: Option<serde_json::Value>,
}

impl ProfileViewDetailed {
    pub fn display_name(&self) -> Option<&str> {
        if let Some(dn) = &self.display_name {
            if dn.len() > 0 {
                Some(dn)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAssociated {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_gens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starter_packs: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labeler: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<ProfileAssociatedChat>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProfileAssociatedChatAllowIncoming {
    All,
    None,
    Following,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAssociatedChat {
    pub allow_incoming: ProfileAssociatedChatAllowIncoming,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted_by_list: Option<defs::bsky::graph::defs::ListViewBasic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_by_list: Option<defs::bsky::graph::defs::ListViewBasic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_followers: Option<KnownFollowers>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KnownFollowers {
    pub count: usize,
    pub followers: Vec<ProfileViewBasic>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "$type")]
pub enum Preference {
    #[serde(rename = "app.bsky.actor.defs#adultContentPref")]
    AdultContentPref(AdultContentPref),
    #[serde(rename = "app.bsky.actor.defs#contentLabelPref")]
    ContentLabelPref(ContentLabelPref),
    #[serde(rename = "app.bsky.actor.defs#savedFeedsPref")]
    SavedFeedsPref(SavedFeedsPref),
    #[serde(rename = "app.bsky.actor.defs#savedFeedsPrefV2")]
    SavedFeedsPrefV2(SavedFeedsPrefV2),
    #[serde(rename = "app.bsky.actor.defs#personalDetailsPref")]
    PersonalDetailsPref(PersonalDetailsPref),
    #[serde(rename = "app.bsky.actor.defs#feedViewPref")]
    FeedViewPref(FeedViewPref),
    #[serde(rename = "app.bsky.actor.defs#threadViewPref")]
    ThreadViewPref(ThreadViewPref),
    #[serde(rename = "app.bsky.actor.defs#interestsPref")]
    InterestsPref(InterestsPref),
    #[serde(rename = "app.bsky.actor.defs#mutedWordsPref")]
    MutedWordsPref(MutedWordsPref),
    #[serde(rename = "app.bsky.actor.defs#hiddenPostsPref")]
    HiddenPostsPref(HiddenPostsPref),
    #[serde(rename = "app.bsky.actor.defs#bskyAppStatePref")]
    BskyAppStatePref(BskyAppStatePref),
    #[serde(rename = "app.bsky.actor.defs#labelersPref")]
    LabelersPref(LabelersPref),
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdultContentPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentLabelPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFeedsPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SavedFeedType {
    Feed,
    List,
    Timeline,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct SavedFeed {
    pub id: String, 
    pub r#type: SavedFeedType,
    pub value: String,
    pub pinned: bool,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFeedsPrefV2 {
    pub items: Vec<SavedFeed>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalDetailsPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadViewPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestsPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutedWordsPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HiddenPostsPref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyAppStatePref(serde_json::Value);

#[derive(std::fmt::Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelersPref(serde_json::Value);
