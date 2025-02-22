use crate::fl;
use crate::models::account::{Account, LinkdingAccountApiResponse};
use crate::models::bookmarks::{
    Bookmark, CheckDetailsResponse, DetailedResponse, LinkdingBookmarksApiCheckResponse,
    LinkdingBookmarksApiResponse,
};
use crate::utils::json::parse_serde_json_value_to_raw_string;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder, StatusCode,
};
use serde_json::Value;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use urlencoding::encode;

pub async fn fetch_bookmarks_from_all_accounts(accounts: Vec<Account>) -> Vec<DetailedResponse> {
    let mut all_responses: Vec<DetailedResponse> = Vec::new();
    for account in accounts {
        match fetch_bookmarks_for_account(&account).await {
            Ok(new_response) => {
                all_responses.push(new_response);
            }
            Err(e) => {
                let epoch_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("")
                    .as_secs();
                #[allow(clippy::cast_possible_wrap)]
                let error_response =
                    DetailedResponse::new(account, epoch_timestamp as i64, false, None);
                all_responses.push(error_response);
                log::error!("Error fetching bookmarks: {}", e);
            }
        }
    }
    all_responses
}

//  NOTE: (vkhitrin) perhaps this method should be split into two:
//        (1) fetch bookmarks (2) fetch archived bookmarks
#[allow(clippy::too_many_lines)]
pub async fn fetch_bookmarks_for_account(
    account: &Account,
) -> Result<DetailedResponse, Box<dyn std::error::Error>> {
    let mut detailed_response = DetailedResponse::new(account.clone(), 0, false, None);
    let mut bookmarks: Vec<Bookmark> = Vec::new();
    let mut headers = HeaderMap::new();
    let rest_api_bookmarks_url: String = account.instance.clone() + "/api/bookmarks/";
    let rest_api_archived_bookmarks_url: String =
        account.instance.clone() + "/api/bookmarks/archived/";
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
                    );
                    bookmarks.push(transformed_bookmark);
                }
            }
            Err(e) => {
                log::error!("Error parsing JSON: {:?}", e);
            }
        }
    } else {
        log::error!(
            "HTTP Error while fetching bookmarks {:?}:\n{:?}",
            bookmarks_response.status(),
            bookmarks_response.text().await
        );
    }
    let archived_bookmarks_response: reqwest::Response = http_client
        .get(rest_api_archived_bookmarks_url)
        .headers(headers)
        .send()
        .await?;
    // NOTE: (vkhitrin) if no Date header was returned, we will use current time.
    let archived_bookmarks_parsed_date = archived_bookmarks_response
        .headers()
        .get("Date")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Utc::now().to_rfc2822()).expect(""));
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
                    );
                    bookmarks.push(transformed_bookmark);
                }
            }
            Err(e) => {
                log::error!("Error parsing JSON: {:?}", e);
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
    detailed_response.bookmarks = Some(bookmarks);
    Ok(detailed_response)
}

#[allow(clippy::too_many_lines)]
pub async fn add_bookmark(
    account: &Account,
    bookmark: &Bookmark,
) -> Result<CheckDetailsResponse, Box<dyn std::error::Error>> {
    let rest_api_url: String = account.instance.clone() + "/api/bookmarks/";
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
        obj.remove("linkding_internal_id");
        obj.remove("website_title");
        obj.remove("website_description");
        obj.remove("web_archive_snapshot_url");
        obj.remove("favicon_url");
        obj.remove("preview_image_url");
        obj.remove("date_added");
        obj.remove("date_modified");
    }
    // NOTE: (vkhitrin) I was not able to get serde_json::value:RawValue to omit quotes
    //let bookmark_url = transformed_json_value["url"].to_string().replace('"', "");
    let bookmark_url =
        parse_serde_json_value_to_raw_string(transformed_json_value.get("url").unwrap());
    match check_bookmark_on_instance(account, bookmark_url.to_string()).await {
        Ok(check) => {
            let metadata = check.metadata;
            if check.bookmark.is_some() {
                let mut bkmrk = check.bookmark.unwrap();
                bkmrk.linkding_internal_id = bkmrk.id;
                bkmrk.user_account_id = account.id;
                bkmrk.id = None;
                if let Some(obj) = transformed_json_value.as_object() {
                    bkmrk.title = match parse_serde_json_value_to_raw_string(
                        transformed_json_value.get("title").unwrap(),
                    ) {
                        ref s if !s.is_empty() => s.to_string(),
                        _ => metadata.title.unwrap(),
                    };
                    bkmrk.description = match parse_serde_json_value_to_raw_string(
                        transformed_json_value.get("description").unwrap(),
                    ) {
                        ref s if !s.is_empty() => s.to_string(),
                        _ => metadata.description.unwrap_or_default(),
                    };
                    bkmrk.notes = match parse_serde_json_value_to_raw_string(
                        transformed_json_value.get("notes").unwrap(),
                    ) {
                        ref s if !s.is_empty() => s.to_string(),
                        _ => String::new(),
                    };
                    bkmrk.tag_names = if let Value::Array(arr) = &obj["tag_names"] {
                        let tags: Vec<String> = arr
                            .iter()
                            .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
                            .collect();
                        tags
                    } else {
                        Vec::new()
                    }
                }
                match edit_bookmark(account, &bkmrk).await {
                    Ok(value) => Ok(CheckDetailsResponse {
                        bookmark: value,
                        is_new: false,
                    }),
                    Err(_e) => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        fl!("failed-to-parse-response"),
                    ))),
                }
            } else {
                let response: reqwest::Response = http_client
                    .post(rest_api_url)
                    .headers(headers)
                    .json(&transformed_json_value)
                    .send()
                    .await?;

                match response.status() {
                    StatusCode::CREATED => match response.json::<Bookmark>().await {
                        Ok(mut value) => {
                            value.linkding_internal_id = value.id;
                            value.user_account_id = account.id;
                            value.id = None;
                            Ok(CheckDetailsResponse {
                                bookmark: value,
                                is_new: true,
                            })
                        }
                        Err(_e) => Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            fl!("failed-to-parse-response"),
                        ))),
                    },
                    status => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        fl!(
                            "http-error",
                            http_rc = status.to_string(),
                            http_err = response.text().await.unwrap()
                        ),
                    ))),
                }
            }
        }
        Err(_e) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!("failed-to-parse-response"),
        ))),
    }
}

pub async fn remove_bookmark(
    account: &Account,
    bookmark: &Bookmark,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/{:?}/",
        account.instance.clone(),
        bookmark.linkding_internal_id.unwrap()
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
        .await?;
    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),
        status => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!(
                "http-error",
                http_rc = status.to_string(),
                http_err = response.text().await.unwrap()
            ),
        ))),
    }
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
        bookmark.linkding_internal_id.unwrap()
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
        obj.remove("linkding_internal_id");
        obj.remove("website_title");
        obj.remove("website_description");
        obj.remove("web_archive_snapshot_url");
        obj.remove("favicon_url");
        obj.remove("preview_image_url");
        obj.remove("date_added");
        obj.remove("date_modified");
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
                value.linkding_internal_id = value.id;
                value.user_account_id = account.id;
                value.id = None;
                Ok(value)
            }
            Err(_e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                fl!("failed-to-parse-response"),
            ))),
        },
        status => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!(
                "http-error",
                http_rc = status.to_string(),
                http_err = response.text().await.unwrap()
            ),
        ))),
    }
}

pub async fn fetch_account_details(account: Account) -> Option<LinkdingAccountApiResponse> {
    let mut account_details: Option<LinkdingAccountApiResponse> = None;
    match check_account_on_instance(&account).await {
        Ok(details) => {
            account_details = Some(details);
        }
        Err(e) => {
            log::error!(
                "Error fetching account {} details: {}",
                account.display_name,
                e
            );
        }
    }
    account_details
}

pub async fn check_account_on_instance(
    account: &Account,
) -> Result<LinkdingAccountApiResponse, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    write!(&mut rest_api_url, "{}/api/user/profile/", account.instance).unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
            Err(_e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                fl!("failed-to-find-linkding-api-endpoint"),
            ))),
        },
        StatusCode::UNAUTHORIZED => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!("invalid-api-token"),
        ))),
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!(
                "unexpected-http-return-code",
                http_rc = response.status().to_string()
            ),
        ))),
    }
}

pub async fn check_bookmark_on_instance(
    account: &Account,
    url: String,
) -> Result<LinkdingBookmarksApiCheckResponse, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = String::new();
    let encoded_bookmark_url = encode(&url);
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/check/?url={}",
        account.instance, encoded_bookmark_url
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
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
            Err(_e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                fl!("failed-to-find-linkding-api-endpoint"),
            ))),
        },
        StatusCode::UNAUTHORIZED => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!("invalid-api-token"),
        ))),
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            fl!(
                "unexpected-http-return-code",
                http_rc = response.status().to_string()
            ),
        ))),
    }
}
