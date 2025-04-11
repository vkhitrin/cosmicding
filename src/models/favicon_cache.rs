use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Eq, PartialEq)]
pub struct Favicon {
    pub url: String,
    pub data: Vec<u8>,
    pub last_sync_timestamp: i64,
}
impl Favicon {
    #[allow(clippy::too_many_arguments)]
    pub fn new(favicon_url: String, favicon_data: Vec<u8>, last_sync_timestamp: i64) -> Self {
        Self {
            url: favicon_url,
            data: favicon_data,
            last_sync_timestamp,
        }
    }
}
