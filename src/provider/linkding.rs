use crate::{
    fl,
    models::{
        account::{Account, LinkdingAccountApiResponse},
        bookmarks::{
            Bookmark, BookmarkCheckDetailsResponse, BookmarkRemoveResponse, DetailedResponse,
            LinkdingBookmarksApiCheckResponse, LinkdingBookmarksApiResponse,
        },
    },
    utils::json::parse_serde_json_value_to_raw_string,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use cosmic::iced_core::image::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder, StatusCode,
};
use serde_json::Value;
use std::{
    fmt::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use urlencoding::encode;

//  NOTE: (vkhitrin) perhaps this method should be split into three:
//        (1) fetch bookmarks
//        (2) fetch archived bookmarks
//        (3) fetch shared
#[allow(clippy::too_many_lines)]
#[allow(clippy::if_same_then_else)]
pub async fn fetch_bookmarks_for_account(
    account: &Account,
) -> Result<DetailedResponse, Box<dyn std::error::Error>> {
    let mut detailed_response = DetailedResponse::new(account.clone(), 0, false, None);
    let mut bookmarks: Vec<Bookmark> = Vec::new();
    let mut headers = HeaderMap::new();
    let rest_api_bookmarks_url: String = account.instance.clone() + "/api/bookmarks/";
    let rest_api_archived_bookmarks_url: String =
        account.instance.clone() + "/api/bookmarks/archived/";
    let rest_api_shared_bookmarks_url: String = account.instance.clone() + "/api/bookmarks/shared/";
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()
        .unwrap();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let bookmarks_response: reqwest::Response = http_client
        .get(rest_api_bookmarks_url)
        .headers(headers.clone())
        .send()
        .await?;
    // NOTE: (vkhitrin) if no Date header was returned, we will use current time.
    let bookmarks_parsed_date = bookmarks_response
        .headers()
        .get("Date")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Utc::now().to_rfc2822()).expect(""));
    if bookmarks_response.status().is_success() {
        detailed_response.successful = true;
        let date: DateTime<Utc> =
            DateTime::parse_from_rfc2822(bookmarks_parsed_date.to_str().unwrap())?
                .with_timezone(&Utc);
        let unix_timestamp = SystemTime::from(date).duration_since(UNIX_EPOCH)?.as_secs();
        detailed_response.timestamp = unix_timestamp as i64;
        let bookmarks_response_json = bookmarks_response
            .json::<LinkdingBookmarksApiResponse>()
            .await;
        // Handle the Result
        match bookmarks_response_json {
            Ok(bookmarks_response) => {
                for bookmark in bookmarks_response.results {
                    let transformed_bookmark = Bookmark::new(
                        account.id,
                        bookmark.id,
                        bookmark.url,
                        bookmark.title,
                        bookmark.description,
                        bookmark.website_title.unwrap_or_else(String::new),
                        bookmark.website_description.unwrap_or_else(String::new),
                        bookmark.notes,
                        bookmark.web_archive_snapshot_url,
                        bookmark.favicon_url.unwrap_or_else(String::new),
                        bookmark.preview_image_url.unwrap_or_else(String::new),
                        bookmark.is_archived,
                        bookmark.unread,
                        bookmark.shared,
                        bookmark.tag_names,
                        bookmark.date_added,
                        bookmark.date_modified,
                        Some(true),
                    );
                    bookmarks.push(transformed_bookmark);
                }
            }
            Err(e) => {
                log::error!("Error parsing JSON: {e:?}");
            }
        }
    } else {
        log::error!(
            "HTTP Error while fetching bookmarks {:?}:\n{:?}",
            bookmarks_response.status(),
            bookmarks_response.text().await
        );
    }
    tokio::time::sleep(Duration::from_millis(0)).await;
    let archived_bookmarks_response: reqwest::Response = http_client
        .get(rest_api_archived_bookmarks_url)
        .headers(headers.clone())
        .send()
        .await?;
    // NOTE: (vkhitrin) if no Date header was returned, we will use current time.
    let archived_bookmarks_parsed_date = archived_bookmarks_response
        .headers()
        .get("Date")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Utc::now().to_rfc2822()).expect(""));
    tokio::time::sleep(Duration::from_millis(0)).await;
    if archived_bookmarks_response.status().is_success() {
        detailed_response.successful = true;
        let date: DateTime<Utc> =
            DateTime::parse_from_rfc2822(archived_bookmarks_parsed_date.to_str().unwrap())?
                .with_timezone(&Utc);
        let unix_timestamp = SystemTime::from(date).duration_since(UNIX_EPOCH)?.as_secs();
        detailed_response.timestamp = unix_timestamp as i64;
        let archived_bookmarks_response_json = archived_bookmarks_response
            .json::<LinkdingBookmarksApiResponse>()
            .await;
        // Handle the Result
        match archived_bookmarks_response_json {
            Ok(archived_bookmarks_response) => {
                for bookmark in archived_bookmarks_response.results {
                    let transformed_bookmark = Bookmark::new(
                        account.id,
                        bookmark.id,
                        bookmark.url,
                        bookmark.title,
                        bookmark.description,
                        bookmark.website_title.unwrap_or_else(String::new),
                        bookmark.website_description.unwrap_or_else(String::new),
                        bookmark.notes,
                        bookmark.web_archive_snapshot_url,
                        bookmark.favicon_url.unwrap_or_else(String::new),
                        bookmark.preview_image_url.unwrap_or_else(String::new),
                        bookmark.is_archived,
                        bookmark.unread,
                        bookmark.shared,
                        bookmark.tag_names,
                        bookmark.date_added,
                        bookmark.date_modified,
                        Some(true),
                    );
                    bookmarks.push(transformed_bookmark);
                }
            }
            Err(e) => {
                log::error!("Error parsing JSON: {e:?}");
            }
        }
    } else {
        detailed_response.successful = false;
        log::error!(
            "HTTP Error while fetching archived bookmarks {:?}:\n{:?}",
            archived_bookmarks_response.status(),
            archived_bookmarks_response.text().await
        );
    }
    tokio::time::sleep(Duration::from_millis(0)).await;
    let shared_bookmarks_response: reqwest::Response = http_client
        .get(rest_api_shared_bookmarks_url)
        .headers(headers)
        .send()
        .await?;
    // NOTE: (vkhitrin) if no Date header was returned, we will use current time.
    let shared_bookmarks_parsed_date = shared_bookmarks_response
        .headers()
        .get("Date")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Utc::now().to_rfc2822()).expect(""));
    if shared_bookmarks_response.status().is_success() {
        detailed_response.successful = true;
        let date: DateTime<Utc> =
            DateTime::parse_from_rfc2822(shared_bookmarks_parsed_date.to_str().unwrap())?
                .with_timezone(&Utc);
        let unix_timestamp = SystemTime::from(date).duration_since(UNIX_EPOCH)?.as_secs();
        detailed_response.timestamp = unix_timestamp as i64;
        let shared_bookmarks_response_json = shared_bookmarks_response
            .json::<LinkdingBookmarksApiResponse>()
            .await;
        // Handle the Result
        match shared_bookmarks_response_json {
            Ok(shared_bookmarks_response) => {
                for bookmark in shared_bookmarks_response.results {
                    let transformed_bookmark = Bookmark::new(
                        account.id,
                        bookmark.id,
                        bookmark.url,
                        bookmark.title,
                        bookmark.description,
                        bookmark.website_title.unwrap_or_else(String::new),
                        bookmark.website_description.unwrap_or_else(String::new),
                        bookmark.notes,
                        bookmark.web_archive_snapshot_url,
                        bookmark.favicon_url.unwrap_or_else(String::new),
                        bookmark.preview_image_url.unwrap_or_else(String::new),
                        bookmark.is_archived,
                        bookmark.unread,
                        bookmark.shared,
                        bookmark.tag_names,
                        bookmark.date_added,
                        bookmark.date_modified,
                        Some(false),
                    );
                    // NOTE: Do not populate bookmarks if they originate from the same
                    // account based on linkding internal bookmark ID.
                    // if !bookmarks.contains(&transformed_bookmark) {
                    if !bookmarks.iter().any(|b| {
                        b.provider_internal_id == transformed_bookmark.provider_internal_id
                    }) {
                        bookmarks.push(transformed_bookmark);
                    } else if bookmarks.is_empty() {
                        bookmarks.push(transformed_bookmark);
                    }
                }
            }
            Err(e) => {
                log::error!("Error parsing JSON: {e:?}");
            }
        }
    } else {
        detailed_response.successful = false;
        log::error!(
            "HTTP Error while fetching archived bookmarks {:?}:\n{:?}",
            shared_bookmarks_response.status(),
            shared_bookmarks_response.text().await
        );
    }
    detailed_response.bookmarks = Some(bookmarks);
    Ok(detailed_response)
}

#[allow(clippy::too_many_lines)]
// NOTE: (vkhitrin) A single method that checks for existence, adds, or updates the bookmark.
pub async fn populate_bookmark(
    account: Account,
    bookmark: Bookmark,
    check_for_existing: bool,
    disable_scraping: bool,
) -> Option<BookmarkCheckDetailsResponse> {
    let rest_api_url: String = account.instance.clone() + "/api/bookmarks/";
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()
        .unwrap();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let mut api_response = BookmarkCheckDetailsResponse::default();
    let mut transformed_json_value: Value = serde_json::to_value(bookmark.clone()).unwrap();
    if let Some(obj) = transformed_json_value.as_object_mut() {
        obj.remove("id");
        obj.remove("user_account_id");
        obj.remove("provider_internal_id");
        obj.remove("website_title");
        obj.remove("website_description");
        obj.remove("web_archive_snapshot_url");
        obj.remove("favicon_url");
        obj.remove("preview_image_url");
        obj.remove("date_added");
        obj.remove("date_modified");
        obj.remove("is_owner");
    }
    // NOTE: (vkhitrin) I was not able to get serde_json::value:RawValue to omit quotes
    //let bookmark_url = transformed_json_value["url"].to_string().replace('"', "");
    let bookmark_url =
        parse_serde_json_value_to_raw_string(transformed_json_value.get("url").unwrap());
    if check_for_existing {
        match check_bookmark_on_instance(&account, bookmark_url.clone(), disable_scraping).await {
            Ok(check) => {
                let metadata = check.metadata;
                if check.bookmark.is_some() {
                    let mut bkmrk = check.bookmark.unwrap();
                    bkmrk.provider_internal_id = bkmrk.id;
                    bkmrk.user_account_id = account.id;
                    bkmrk.id = None;
                    if let Some(obj) = transformed_json_value.as_object() {
                        bkmrk.title = match parse_serde_json_value_to_raw_string(
                            transformed_json_value.get("title").unwrap(),
                        ) {
                            ref s if !s.is_empty() => s.clone(),
                            _ => metadata.title.unwrap(),
                        };
                        bkmrk.description = match parse_serde_json_value_to_raw_string(
                            transformed_json_value.get("description").unwrap(),
                        ) {
                            ref s if !s.is_empty() => s.clone(),
                            _ => metadata.description.unwrap_or_default(),
                        };
                        bkmrk.notes = match parse_serde_json_value_to_raw_string(
                            transformed_json_value.get("notes").unwrap(),
                        ) {
                            ref s if !s.is_empty() => s.clone(),
                            _ => String::new(),
                        };
                        bkmrk.tag_names = if let Value::Array(arr) = &obj["tag_names"] {
                            let tags: Vec<String> = arr
                                .iter()
                                .filter_map(|item| {
                                    item.as_str().map(std::string::ToString::to_string)
                                })
                                .collect();
                            tags
                        } else {
                            Vec::new()
                        }
                    }
                    api_response.bookmark = Some(bkmrk);
                } else {
                    api_response.is_new = true;
                }
            }
            Err(_e) => api_response.error = Some(fl!("failed-to-parse-response")),
        }
    } else {
        api_response.bookmark = Some(bookmark);
    }
    if api_response.is_new {
        let max_retries = 3;
        let mut retry_count = 0;
        let mut last_error: Option<String> = None;

        while retry_count <= max_retries {
            if retry_count > 0 {
                let backoff_ms = 1000 * u64::pow(2, retry_count - 1);
                log::warn!(
                    "Retrying bookmark creation (attempt {}/{}) after {}ms backoff",
                    retry_count,
                    max_retries,
                    backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }

            let response_result = http_client
                .post(&rest_api_url)
                .headers(headers.clone())
                .json(&transformed_json_value)
                .send()
                .await;

            match response_result {
                Ok(response) => match response.status() {
                    StatusCode::CREATED => match response.json::<Bookmark>().await {
                        Ok(mut value) => {
                            value.provider_internal_id = value.id;
                            value.user_account_id = account.id;
                            value.id = None;
                            api_response.bookmark = Some(value);
                            api_response.successful = true;
                            break;
                        }
                        Err(_e) => {
                            api_response.error = Some(fl!("failed-to-parse-response"));
                            break;
                        }
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        let error_msg = response.text().await.unwrap_or_default();
                        last_error = Some(fl!(
                            "http-error",
                            http_rc = StatusCode::SERVICE_UNAVAILABLE.to_string(),
                            http_err = error_msg
                        ));
                        log::error!(
                            "Error adding bookmark (503): {}",
                            last_error.as_ref().unwrap()
                        );
                        retry_count += 1;
                    }
                    status => {
                        api_response.error = Some(fl!(
                            "http-error",
                            http_rc = status.to_string(),
                            http_err = response.text().await.unwrap_or_default()
                        ));
                        log::error!(
                            "Error adding bookmark: {}",
                            api_response.error.as_ref().unwrap()
                        );
                        break;
                    }
                },
                Err(e) => {
                    api_response.error = Some(format!("Request failed: {e}"));
                    log::error!("Error sending request: {e}");
                    break;
                }
            }
        }

        if retry_count > max_retries && api_response.error.is_none() {
            api_response.error = last_error;
        }
    } else if let Some(bookmark) = &api_response.bookmark {
        match edit_bookmark(&account, bookmark).await {
            Ok(value) => {
                api_response.bookmark = Some(value);
                api_response.successful = true;
            }
            Err(_e) => api_response.error = Some(fl!("failed-to-parse-response")),
        }
    } else {
        api_response.error = Some(fl!("failed-to-parse-response"));
    }
    Some(api_response)
}

pub async fn remove_bookmark(
    account: Account,
    bookmark: Bookmark,
) -> Option<BookmarkRemoveResponse> {
    let mut api_response: BookmarkRemoveResponse = BookmarkRemoveResponse::default();
    let mut rest_api_url: String = String::new();
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/{:?}/",
        account.instance.clone(),
        bookmark.provider_internal_id.unwrap()
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()
        .unwrap();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let response: reqwest::Response = http_client
        .delete(rest_api_url)
        .headers(headers)
        .send()
        .await
        .unwrap();
    match response.status() {
        StatusCode::NO_CONTENT => {
            api_response.successful = true;
        }
        status => {
            api_response.error = Some(fl!(
                "http-error",
                http_rc = status.to_string(),
                http_err = response.text().await.unwrap()
            ));
            log::error!(
                "Error removing bookmark: {}",
                api_response.error.as_ref().unwrap()
            );
        }
    }
    Some(api_response)
}

pub async fn edit_bookmark(
    account: &Account,
    bookmark: &Bookmark,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/{:?}/",
        account.instance.clone(),
        bookmark.provider_internal_id.unwrap()
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()
        .unwrap();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let mut transformed_json_value: Value = serde_json::to_value(bookmark)?;
    if let Some(obj) = transformed_json_value.as_object_mut() {
        obj.remove("id");
        obj.remove("user_account_id");
        obj.remove("provider_internal_id");
        obj.remove("website_title");
        obj.remove("website_description");
        obj.remove("web_archive_snapshot_url");
        obj.remove("favicon_url");
        obj.remove("preview_image_url");
        obj.remove("date_added");
        obj.remove("date_modified");
        obj.remove("is_owner");
    }
    let response: reqwest::Response = http_client
        .patch(rest_api_url)
        .headers(headers)
        .json(&transformed_json_value)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => match response.json::<Bookmark>().await {
            Ok(mut value) => {
                value.provider_internal_id = value.id;
                value.user_account_id = account.id;
                value.id = None;
                value.is_owner = Some(true);
                Ok(value)
            }
            Err(_e) => {
                log::error!("Failed to parse response");
                Err(Box::new(std::io::Error::other(fl!(
                    "failed-to-parse-response"
                ))))
            }
        },
        status => {
            let http_rc = status.to_string();
            let http_err = response.text().await.unwrap();
            log::error!("HTTP Error: {http_rc} {http_err}");
            Err(Box::new(std::io::Error::other(fl!(
                "http-error",
                http_rc = http_rc,
                http_err = http_err
            ))))
        }
    }
}

pub async fn fetch_account_details(account: Account) -> Option<LinkdingAccountApiResponse> {
    let mut account_details: Option<LinkdingAccountApiResponse> =
        Some(LinkdingAccountApiResponse::default());
    match check_account_on_instance(&account).await {
        Ok(mut details) => {
            details.successful = Some(true);
            account_details = Some(details);
        }
        Err(e) => {
            account_details.as_mut().unwrap().successful = Some(false);
            if e.to_string().contains("builder error") {
                account_details.as_mut().unwrap().error = Some(fl!("provided-url-is-not-valid"));
            } else {
                account_details.as_mut().unwrap().error = Some(e.to_string());
            }

            log::error!(
                "Error fetching account {} details: {}",
                account.display_name,
                e
            );
        }
    }
    account_details
}

/// Get provider version for linkding from API response
pub fn get_provider_version(api_response: &LinkdingAccountApiResponse) -> Option<String> {
    api_response.version.clone()
}

pub async fn check_account_on_instance(
    account: &Account,
) -> Result<LinkdingAccountApiResponse, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    write!(&mut rest_api_url, "{}/api/user/profile/", account.instance).unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()?;
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let response: reqwest::Response = http_client
        .get(rest_api_url)
        .headers(headers)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => match response.json::<LinkdingAccountApiResponse>().await {
            Ok(value) => Ok(value),
            Err(_e) => Err(Box::new(std::io::Error::other(fl!(
                "failed-to-find-linkding-api-endpoint"
            )))),
        },
        StatusCode::UNAUTHORIZED => Err(Box::new(std::io::Error::other(fl!("invalid-api-token")))),
        _ => Err(Box::new(std::io::Error::other(fl!(
            "unexpected-http-return-code",
            http_rc = response.status().to_string()
        )))),
    }
}

pub async fn check_bookmark_on_instance(
    account: &Account,
    url: String,
    disable_scraping: bool,
) -> Result<LinkdingBookmarksApiCheckResponse, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    let encoded_bookmark_url = encode(&url);

    if disable_scraping {
        write!(
            &mut rest_api_url,
            "{}/api/bookmarks/check/?url={}&disable_scraping=true",
            account.instance, encoded_bookmark_url
        )
        .unwrap();
    } else {
        write!(
            &mut rest_api_url,
            "{}/api/bookmarks/check/?url={}",
            account.instance, encoded_bookmark_url
        )
        .unwrap();
    }
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.trust_invalid_certs)
        .build()?;
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let response: reqwest::Response = http_client
        .get(rest_api_url)
        .headers(headers)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => match response.json::<LinkdingBookmarksApiCheckResponse>().await {
            Ok(value) => Ok(value),
            Err(_e) => Err(Box::new(std::io::Error::other(fl!(
                "failed-to-find-linkding-api-endpoint"
            )))),
        },
        StatusCode::UNAUTHORIZED => Err(Box::new(std::io::Error::other(fl!("invalid-api-token")))),
        _ => Err(Box::new(std::io::Error::other(fl!(
            "unexpected-http-return-code",
            http_rc = response.status().to_string()
        )))),
    }
}

pub async fn fetch_bookmark_favicon(url: String) -> Bytes {
    let mut bytes: Bytes = Bytes::new();
    let http_client = ClientBuilder::new()
        .build()
        .expect("Failed to construct HTTP client");
    let response: reqwest::Response = http_client
        .get(url)
        .send()
        .await
        .expect("Failed fetching favicon");
    match response.status() {
        StatusCode::OK => match response.bytes().await {
            Ok(value) => bytes = value,
            Err(e) => {
                log::error!("Error fetching favicon: {e}");
            }
        },
        _ => log::error!(
            "Unexpected http return code {:?}",
            response.status().to_string()
        ),
    }
    bytes
}
