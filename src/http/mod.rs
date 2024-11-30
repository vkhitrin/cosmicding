use crate::fl;
use crate::models::account::{Account, AccountApiResponse};
use crate::models::bookmarks::{Bookmark, BookmarksApiResponse, DetailedResponse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder, StatusCode,
};
use serde_json::Value;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

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
                let error_response =
                    DetailedResponse::new(account, epoch_timestamp as i64, false, None);
                all_responses.push(error_response);
                log::error!("Error fetching bookmarks: {}", e);
            }
        }
    }
    return all_responses;
}

pub async fn fetch_bookmarks_for_account(
    account: &Account,
) -> Result<DetailedResponse, Box<dyn std::error::Error>> {
    let mut detailed_response = DetailedResponse::new(account.clone(), 0, false, None);
    let mut bookmarks: Vec<Bookmark> = Vec::new();
    let mut headers = HeaderMap::new();
    let rest_api_url: String = account.instance.clone() + "/api/bookmarks/";
    let http_client = ClientBuilder::new()
        .danger_accept_invalid_certs(account.tls)
        .build()
        .unwrap();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", account.api_token)).unwrap(),
    );
    let response: reqwest::Response = http_client
        .get(rest_api_url)
        .headers(headers)
        .send()
        .await?;
    let parsed_date = response.headers().get("Date").unwrap().to_str().unwrap();
    if response.status().is_success() {
        detailed_response.successful = true;
        let date: DateTime<Utc> = DateTime::parse_from_rfc2822(parsed_date)?.with_timezone(&Utc);
        let unix_timestamp = SystemTime::from(date).duration_since(UNIX_EPOCH)?.as_secs();
        detailed_response.timestamp = unix_timestamp as i64;
        let response_json = response.json::<BookmarksApiResponse>().await;
        // Handle the Result
        match response_json {
            Ok(bookmarks_response) => {
                for bookmark in bookmarks_response.results {
                    let transformed_bookmark = Bookmark::new(
                        account.id,
                        bookmark.id,
                        bookmark.url,
                        bookmark.title,
                        bookmark.description,
                        bookmark.website_title.unwrap_or_else(|| "".to_string()),
                        bookmark
                            .website_description
                            .unwrap_or_else(|| "".to_string()),
                        bookmark.notes,
                        bookmark.web_archive_snapshot_url,
                        bookmark.favicon_url.unwrap_or_else(|| "".to_string()),
                        bookmark.preview_image_url.unwrap_or_else(|| "".to_string()),
                        bookmark.is_archived,
                        bookmark.unread,
                        bookmark.shared,
                        bookmark.tag_names,
                        bookmark.date_added,
                        bookmark.date_modified,
                    );
                    bookmarks.push(transformed_bookmark);
                }
                detailed_response.bookmarks = Some(bookmarks);
            }
            Err(e) => {
                log::error!("Error parsing JSON: {:?}", e);
            }
        }
    } else {
        log::error!(
            "HTTP Error {:?}:\n{:?}",
            response.status(),
            response.text().await
        );
    }
    Ok(detailed_response)
}

pub async fn add_bookmark(
    account: &Account,
    bookmark: &Bookmark,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
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
    let mut transformed_json_value: Value = serde_json::to_value(&bookmark)?;
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

pub async fn remove_bookmark(
    account: &Account,
    bookmark: &Bookmark,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rest_api_url: String = "".to_owned();
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/{:?}/",
        account.instance.clone(),
        bookmark.linkding_internal_id.clone().unwrap()
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
    let mut rest_api_url: String = "".to_owned();
    write!(
        &mut rest_api_url,
        "{}/api/bookmarks/{:?}/",
        account.instance.clone(),
        bookmark.linkding_internal_id.clone().unwrap()
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
    let mut transformed_json_value: Value = serde_json::to_value(&bookmark)?;
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

pub async fn fetch_account_details(account: Account) -> Option<AccountApiResponse> {
    let mut account_details: Option<AccountApiResponse> = None;
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
    return account_details;
}

pub async fn check_account_on_instance(
    account: &Account,
) -> Result<AccountApiResponse, Box<dyn std::error::Error>> {
    let mut rest_api_url: String = "".to_owned();
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
        StatusCode::OK => match response.json::<AccountApiResponse>().await {
            Ok(_value) => Ok(_value),
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
