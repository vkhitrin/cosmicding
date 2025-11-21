use crate::models::bookmarks::Bookmark;
use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkFormat {
    Netscape,
}

pub trait BookmarkParser {
    fn parse(&self, content: &str) -> Result<Vec<Bookmark>>;

    fn generate(&self, bookmarks: &[Bookmark]) -> String;
}

pub struct BookmarkIO;

impl BookmarkIO {
    pub fn parse(content: &str, format: BookmarkFormat) -> Result<Vec<Bookmark>> {
        let parser = Self::get_parser(format);
        parser.parse(content)
    }

    pub fn generate(bookmarks: &[Bookmark], format: BookmarkFormat) -> String {
        let parser = Self::get_parser(format);
        parser.generate(bookmarks)
    }

    fn get_parser(format: BookmarkFormat) -> Box<dyn BookmarkParser> {
        match format {
            BookmarkFormat::Netscape => Box::new(NetscapeParser),
        }
    }

    #[allow(dead_code)]
    pub fn detect_format(content: &str) -> Option<BookmarkFormat> {
        if content.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>")
            || (content.contains("<DT><A HREF=") || content.contains("<DT><a href="))
        {
            return Some(BookmarkFormat::Netscape);
        }
        None
    }
}

struct NetscapeParser;

impl BookmarkParser for NetscapeParser {
    fn parse(&self, html_content: &str) -> Result<Vec<Bookmark>> {
        parse_netscape_html(html_content)
    }

    fn generate(&self, bookmarks: &[Bookmark]) -> String {
        generate_netscape_html(bookmarks)
    }
}

/// Parses Netscape HTML bookmark format and returns a list of bookmarks
pub fn parse_netscape_html(html_content: &str) -> Result<Vec<Bookmark>> {
    let mut bookmarks = Vec::new();
    let lines: Vec<&str> = html_content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("<DT><A HREF=") || line.starts_with("<DT><a href=") {
            if let Some(bookmark) = parse_bookmark_entry(line, &lines, &mut i)? {
                bookmarks.push(bookmark);
            }
        }
        i += 1;
    }

    Ok(bookmarks)
}

/// Generates Netscape HTML format from bookmarks
pub fn generate_netscape_html(bookmarks: &[Bookmark]) -> String {
    let mut html = String::new();

    // Header
    html.push_str("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
    html.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
    html.push_str("<TITLE>Bookmarks</TITLE>\n");
    html.push_str("<H1>Bookmarks</H1>\n");
    html.push_str("<DL><p>\n");

    for bookmark in bookmarks {
        html.push_str("    <DT><A HREF=\"");
        html.push_str(&html_escape(&bookmark.url));
        html.push('"');

        if let Some(date_added) = &bookmark.date_added {
            if let Ok(dt) = DateTime::parse_from_rfc3339(date_added) {
                html.push_str(" ADD_DATE=\"");
                html.push_str(&dt.timestamp().to_string());
                html.push('"');
            }
        }

        if !bookmark.tag_names.is_empty() {
            html.push_str(" TAGS=\"");
            html.push_str(&html_escape(&bookmark.tag_names.join(",")));
            html.push('"');
        }

        html.push('>');
        html.push_str(&html_escape(&bookmark.title));
        html.push_str("</A>\n");

        if !bookmark.description.is_empty() {
            html.push_str("    <DD>");
            html.push_str(&html_escape(&bookmark.description));
            html.push('\n');
        }
    }

    html.push_str("</DL><p>\n");

    html
}

fn parse_bookmark_entry(
    line: &str,
    lines: &[&str],
    current_index: &mut usize,
) -> Result<Option<Bookmark>> {
    let attributes = parse_anchor_attributes(line)?;

    let url = attributes
        .get("HREF")
        .or_else(|| attributes.get("href"))
        .ok_or_else(|| anyhow!("Missing HREF attribute"))?
        .clone();

    let title = extract_title_from_anchor(line)?;

    let tag_names = attributes
        .get("TAGS")
        .or_else(|| attributes.get("tags"))
        .map(|tags_str| {
            tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let date_added = attributes
        .get("ADD_DATE")
        .or_else(|| attributes.get("add_date"))
        .and_then(|ts| ts.parse::<i64>().ok())
        .and_then(|ts| {
            Utc.timestamp_opt(ts, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        });

    let mut description = String::new();
    if *current_index + 1 < lines.len() {
        let next_line = lines[*current_index + 1].trim();
        if next_line.starts_with("<DD>") || next_line.starts_with("<dd>") {
            let desc_text = next_line
                .trim_start_matches("<DD>")
                .trim_start_matches("<dd>")
                .trim();
            description = html_unescape(desc_text);
            *current_index += 1; // Skip the description line
        }
    }

    let bookmark = Bookmark {
        id: None,
        user_account_id: None,
        provider_internal_id: None,
        url,
        title,
        description,
        website_title: None,
        website_description: None,
        notes: String::new(),
        web_archive_snapshot_url: String::new(),
        favicon_url: None,
        preview_image_url: None,
        is_archived: false,
        unread: false,
        shared: false,
        tag_names,
        date_added,
        date_modified: None,
        is_owner: None,
        favicon_cached: None,
    };

    Ok(Some(bookmark))
}

/// Extracts attributes from an anchor tag
fn parse_anchor_attributes(line: &str) -> Result<HashMap<String, String>> {
    let mut attributes = HashMap::new();

    let start = line
        .find("<A ")
        .or_else(|| line.find("<a "))
        .ok_or_else(|| anyhow!("No anchor tag found"))?;

    let end = line[start..]
        .find('>')
        .ok_or_else(|| anyhow!("Unclosed anchor tag"))?;

    let attrs_str = &line[start..start + end];

    let chars = attrs_str.chars().peekable();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_quotes = false;
    let mut reading_value = false;

    for ch in chars {
        match ch {
            ' ' | '\t' if !in_quotes => {
                if reading_value {
                    attributes.insert(current_key.clone(), current_value.clone());
                    current_key.clear();
                    current_value.clear();
                    reading_value = false;
                } else {
                    current_key.clear();
                }
            }
            '=' if !in_quotes => {
                reading_value = true;
            }
            '"' => {
                if in_quotes {
                    attributes.insert(current_key.clone(), current_value.clone());
                    current_key.clear();
                    current_value.clear();
                    reading_value = false;
                }
                in_quotes = !in_quotes;
            }
            _ => {
                if in_quotes || reading_value {
                    current_value.push(ch);
                } else if ch.is_alphanumeric() || ch == '_' {
                    current_key.push(ch);
                }
            }
        }
    }

    if !current_key.is_empty() && !current_value.is_empty() {
        attributes.insert(current_key, current_value);
    }

    Ok(attributes)
}

fn extract_title_from_anchor(line: &str) -> Result<String> {
    let anchor_start = line
        .find("<A ")
        .or_else(|| line.find("<a "))
        .ok_or_else(|| anyhow!("No anchor tag found"))?;

    let start = line[anchor_start..]
        .find('>')
        .ok_or_else(|| anyhow!("No closing > in anchor tag"))?
        + anchor_start;

    let end = line
        .rfind("</A>")
        .or_else(|| line.rfind("</a>"))
        .ok_or_else(|| anyhow!("No closing </A> tag"))?;

    let title = line[start + 1..end].trim();
    Ok(html_unescape(title))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn html_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&") // Must be last to avoid double-unescaping
}
