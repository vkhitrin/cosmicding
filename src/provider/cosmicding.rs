use crate::{
    db::SqliteDatabase,
    models::{
        account::Account,
        bookmarks::{
            Bookmark, BookmarkCheckDetailsResponse, BookmarkRemoveResponse, DetailedResponse,
        },
    },
};
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_provider_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn fetch_bookmarks_for_account(account: &Account) -> DetailedResponse {
    let epoch_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("")
        .as_secs();

    #[allow(clippy::cast_possible_wrap)]
    DetailedResponse::new(
        account.clone(),
        epoch_timestamp as i64,
        true,
        Some(Vec::new()),
    )
}

pub async fn populate_bookmark(
    account: Account,
    mut bookmark: Bookmark,
    check_for_existing: bool,
    _disable_scraping: bool,
) -> Option<BookmarkCheckDetailsResponse> {
    bookmark.user_account_id = account.id;

    let mut is_new = bookmark.id.is_none();

    // NOTE: (vkhitrin) Check if a bookmark with the same URL already exists in the local database
    //       Similar logic to linkding's populate_bookmark function which checks the remote API.
    if check_for_existing {
        if let Some(account_id) = account.id {
            if let Ok(mut db) = SqliteDatabase::create().await {
                if let Some(existing_bookmark) =
                    db.find_bookmark_by_url(account_id, &bookmark.url).await
                {
                    is_new = false;
                    bookmark.id = existing_bookmark.id;
                    bookmark.date_added = existing_bookmark.date_added;
                }
            }
        }
    }

    // NOTE: (vkhitrin) Set timestamps for local bookmarks
    //       Use ISO 8601 format with 'Z' suffix to match linkding API format
    let timestamp_string = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();

    if is_new {
        bookmark.date_added = Some(timestamp_string.clone());
        bookmark.date_modified = Some(timestamp_string);
    } else {
        bookmark.date_modified = Some(timestamp_string);
    }

    Some(BookmarkCheckDetailsResponse {
        bookmark: Some(bookmark),
        successful: true,
        is_new,
        ..Default::default()
    })
}

pub async fn remove_bookmark(
    _account: Account,
    _bookmark: Bookmark,
) -> Option<BookmarkRemoveResponse> {
    Some(BookmarkRemoveResponse {
        successful: true,
        ..Default::default()
    })
}
