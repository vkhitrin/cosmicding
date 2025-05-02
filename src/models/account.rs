use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Serialize, Deserialize, Debug, Clone, FromRow, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Account {
    pub api_token: String,
    pub display_name: String,
    pub enable_public_sharing: bool,
    pub enable_sharing: bool,
    pub enabled: bool,
    pub id: Option<i64>,
    pub instance: String,
    pub last_sync_status: bool,
    pub last_sync_timestamp: i64,
    pub trust_invalid_certs: bool,
}

impl AsRef<str> for Account {
    fn as_ref(&self) -> &str {
        &self.display_name
    }
}

impl Account {
    pub fn new(name: String, token: String, url: String) -> Self {
        Self {
            api_token: token,
            display_name: name,
            enable_public_sharing: false,
            enable_sharing: false,
            enabled: true,
            id: None,
            instance: url,
            last_sync_status: false,
            last_sync_timestamp: 0,
            trust_invalid_certs: false,
        }
    }
    pub fn requires_remote_sync(&self, other: &Account) -> bool {
        self.api_token != other.api_token
            || self.enable_public_sharing != other.enable_public_sharing
            || self.enable_sharing != other.enable_sharing
            || self.enabled != other.enabled
            || self.id != other.id
            || self.instance != other.instance
            || self.trust_invalid_certs != other.trust_invalid_certs
    }
}

// NOTE: (vkhitrin) we do not use these preferences as part of the application.
//       This is a response from the API and is not used in the application.
//       We implement a general sorting mechanism for all bookmarks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchPreferences {
    pub sort: Option<String>,
    pub shared: Option<String>,
    pub unread: Option<String>,
}

// NOTE: (vkhitrin) we do not use most of these values as part of the application.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct LinkdingAccountApiResponse {
    pub bookmark_date_display: String,
    pub display_url: bool,
    // NOTE: (vkhitrin) we do not check if the account enabled favicons, we check individual
    // bookmarks if they contain favicon URL
    pub enable_favicons: bool,
    pub enable_public_sharing: bool,
    pub enable_sharing: bool,
    // NOTE: (vkhitrin) internal field to represent a potential failure
    pub error: Option<String>,
    pub permanent_notes: bool,
    pub search_preferences: SearchPreferences,
    // NOTE: (vkhitrin) internal field to represent a successful API call
    pub successful: Option<bool>,
    pub tag_search: String,
    pub theme: String,
    pub web_archive_integration: String,
}
