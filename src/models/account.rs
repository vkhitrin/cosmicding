use sqlx::FromRow;
#[derive(Debug, Clone, FromRow, Eq, PartialEq)]
pub struct Account {
    pub id: Option<i64>,
    pub api_token: String,
    pub display_name: String,
    pub instance: String,
    pub last_sync_status: bool,
    pub last_sync_timestamp: i64,
    pub tls: bool,
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
        }
    }
}
