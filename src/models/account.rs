use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Serialize, Deserialize, Debug, Clone, FromRow, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Account {
    pub id: Option<i64>,
    pub api_token: String,
    pub display_name: String,
    pub instance: String,
    pub last_sync_status: bool,
    pub last_sync_timestamp: i64,
    pub tls: bool,
    pub enable_sharing: bool,
    pub enable_public_sharing: bool,
}

impl AsRef<str> for Account {
    fn as_ref(&self) -> &str {
        &self.display_name
    }
}

impl Account {
    pub fn new(name: String, token: String, url: String) -> Self {
        Self {
            id: None,
            api_token: token,
            display_name: name,
            instance: url,
            last_sync_status: false,
            last_sync_timestamp: 0,
            tls: true,
            enable_sharing: false,
            enable_public_sharing: false,
        }
    }
}

// NOTE: (vkhitrin) we do not use these preferences as part of the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPreferences {
    pub sort: Option<String>,
    pub shared: Option<String>,
    pub unread: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LinkdingAccountApiResponse {
    pub theme: String,
    pub bookmark_date_display: String,
    pub web_archive_integration: String,
    pub tag_search: String,
    pub enable_sharing: bool,
    pub enable_public_sharing: bool,
    pub enable_favicons: bool,
    pub display_url: bool,
    pub permanent_notes: bool,
    pub search_preferences: SearchPreferences,
}
