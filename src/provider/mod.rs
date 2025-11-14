mod cosmicding;
mod linkding;

use crate::models::{
    account::{Account, LinkdingAccountApiResponse},
    bookmarks::{Bookmark, BookmarkCheckDetailsResponse, BookmarkRemoveResponse, DetailedResponse},
    provider::Provider,
};
use cosmic::iced_core::image::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ALLOWED_PROVIDERS: &[&str] = &["Linkding"];

pub async fn fetch_bookmarks_for_single_account(account: Account) -> DetailedResponse {
    match account.provider() {
        Provider::Cosmicding => cosmicding::fetch_bookmarks_for_account(&account).await,
        Provider::Linkding => match linkding::fetch_bookmarks_for_account(&account).await {
            Ok(response) => response,
            Err(e) => {
                let epoch_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();
                #[allow(clippy::cast_possible_wrap)]
                let error_response =
                    DetailedResponse::new(account, epoch_timestamp as i64, false, None);
                log::error!("Error fetching linkding bookmarks: {e}");
                error_response
            }
        },
    }
}

pub async fn populate_bookmark(
    account: Account,
    bookmark: Bookmark,
    check_for_existing: bool,
    disable_scraping: bool,
) -> Option<BookmarkCheckDetailsResponse> {
    match account.provider() {
        Provider::Cosmicding => {
            cosmicding::populate_bookmark(account, bookmark, check_for_existing, disable_scraping)
                .await
        }
        Provider::Linkding => {
            linkding::populate_bookmark(account, bookmark, check_for_existing, disable_scraping)
                .await
        }
    }
}

pub async fn remove_bookmark(
    account: Account,
    bookmark: Bookmark,
) -> Option<BookmarkRemoveResponse> {
    match account.provider() {
        Provider::Cosmicding => cosmicding::remove_bookmark(account, bookmark).await,
        Provider::Linkding => linkding::remove_bookmark(account, bookmark).await,
    }
}

pub async fn fetch_account_details(account: Account) -> Option<LinkdingAccountApiResponse> {
    match account.provider() {
        Provider::Cosmicding => None, // Local provider has no remote account details
        Provider::Linkding => linkding::fetch_account_details(account).await,
    }
}

pub async fn fetch_bookmark_favicon(url: String) -> Bytes {
    linkding::fetch_bookmark_favicon(url).await
}

pub fn get_provider_version(
    provider: Provider,
    api_response: Option<&LinkdingAccountApiResponse>,
) -> Option<String> {
    match provider {
        Provider::Cosmicding => Some(cosmicding::get_provider_version()),
        Provider::Linkding => api_response.and_then(linkding::get_provider_version),
    }
}
