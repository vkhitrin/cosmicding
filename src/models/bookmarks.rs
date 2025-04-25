use crate::models::account::Account;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::favicon_cache::Favicon;

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, Eq, PartialEq)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub user_account_id: Option<i64>,
    pub linkding_internal_id: Option<i64>,
    pub url: String,
    pub title: String,
    pub description: String,
    pub website_title: Option<String>,
    pub website_description: Option<String>,
    pub notes: String,
    pub web_archive_snapshot_url: String,
    pub favicon_url: Option<String>,
    pub preview_image_url: Option<String>,
    pub is_archived: bool,
    pub unread: bool,
    pub shared: bool,
    pub tag_names: Vec<String>,
    pub date_added: Option<String>,
    pub date_modified: Option<String>,
    pub is_owner: Option<bool>,
    pub favicon_cached: Option<Favicon>,
}

// NOTE: (vkhitrin) as of March 1st, 2025, linkding doesn't expose the user which shared the
// bookmark, we will maintain an internal field to indicate if the current account is an owner.
impl Bookmark {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: Option<i64>,
        linkding_id: Option<i64>,
        linkding_url: String,
        linkding_title: String,
        linkding_description: String,
        linkding_website_title: String,
        linkding_website_description: String,
        linkding_notes: String,
        linkding_web_archive_snapshot_url: String,
        linkding_favicon_url: String,
        linkding_preview_image_url: String,
        linkding_is_archived: bool,
        linkding_unread: bool,
        linkding_shared: bool,
        linkding_tag_names: Vec<String>,
        linkding_date_added: Option<String>,
        linkding_date_modified: Option<String>,
        internal_workaround_is_owner: Option<bool>,
    ) -> Self {
        Self {
            id: None,
            user_account_id: account_id,
            linkding_internal_id: linkding_id,
            url: linkding_url,
            title: linkding_title,
            description: linkding_description,
            website_title: Some(linkding_website_title),
            website_description: Some(linkding_website_description),
            notes: linkding_notes,
            web_archive_snapshot_url: linkding_web_archive_snapshot_url,
            favicon_url: Some(linkding_favicon_url),
            preview_image_url: Some(linkding_preview_image_url),
            is_archived: linkding_is_archived,
            unread: linkding_unread,
            shared: linkding_shared,
            tag_names: linkding_tag_names,
            date_added: linkding_date_added,
            date_modified: linkding_date_modified,
            is_owner: internal_workaround_is_owner,
            favicon_cached: None,
        }
    }
    pub fn merge(self, other: Self) -> Self {
        Self {
            id: self.id.or(other.id),
            user_account_id: self.user_account_id.or(other.user_account_id),
            linkding_internal_id: self.linkding_internal_id.or(other.linkding_internal_id),
            url: other.url,
            title: other.title,
            description: other.description,
            website_title: self.website_title.or(other.website_title),
            website_description: self.website_description.or(other.website_description),
            notes: other.notes,
            web_archive_snapshot_url: other.web_archive_snapshot_url,
            favicon_url: self.favicon_url.or(other.favicon_url),
            preview_image_url: self.preview_image_url.or(other.preview_image_url),
            is_archived: other.is_archived,
            unread: other.unread,
            shared: other.shared,
            tag_names: other.tag_names,
            date_added: self.date_added.or(other.date_added),
            date_modified: self.date_modified.or(other.date_modified),
            is_owner: self.is_owner,
            favicon_cached: self.favicon_cached,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkdingBookmarksApiResponse {
    pub count: u64,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<Bookmark>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedResponse {
    pub account: Account,
    pub timestamp: i64,
    pub successful: bool,
    pub bookmarks: Option<Vec<Bookmark>>,
}

impl DetailedResponse {
    pub fn new(
        response_account: Account,
        response_timestamp: i64,
        response_successful: bool,
        response_bookmarks: Option<Vec<Bookmark>>,
    ) -> Self {
        Self {
            account: response_account,
            timestamp: response_timestamp,
            successful: response_successful,
            bookmarks: response_bookmarks,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkdingBookmarksApiCheckMetadata {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub preview_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkdingBookmarksApiCheckResponse {
    pub bookmark: Option<Bookmark>,
    pub metadata: LinkdingBookmarksApiCheckMetadata,
    pub auto_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkCheckDetailsResponse {
    pub bookmark: Option<Bookmark>,
    pub error: Option<String>,
    pub is_new: bool,
    pub successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkRemoveResponse {
    pub error: Option<String>,
    pub successful: bool,
}
