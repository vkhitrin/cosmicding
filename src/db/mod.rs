use crate::app::config::SortOption;
use anyhow::{anyhow, Result};

use std::path::Path;

use sqlx::sqlite::Sqlite;
use sqlx::{migrate::MigrateDatabase, prelude::*, SqlitePool};

use crate::app::{APP, APPID, ORG, QUALIFIER};
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;

const DB_PATH: &str = constcat::concat!(APPID, "-db", ".sqlite");

#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    conn: SqlitePool,
}

impl SqliteDatabase {
    pub async fn create() -> Result<Self> {
        let directories = directories::ProjectDirs::from(QUALIFIER, ORG, APP).unwrap();
        std::fs::create_dir_all(directories.cache_dir())?;
        Self::ensure_db_exists(directories.cache_dir()).await
    }

    async fn ensure_db_exists(db_dir: &Path) -> Result<Self> {
        let db_path = db_dir.join(DB_PATH);

        let db_path = db_path
            .to_str()
            .ok_or(anyhow!("can't convert path to str"))?;

        if !Sqlite::database_exists(db_path).await? {
            Sqlite::create_database(db_path).await?;
        }

        let conn = &SqlitePool::connect(db_path).await?;

        let migration_path = db_dir.join("migrations");
        std::fs::create_dir_all(&migration_path)?;
        include_dir::include_dir!("migrations")
            .extract(&migration_path)
            .unwrap();

        match sqlx::migrate::Migrator::new(migration_path).await {
            Ok(migrator) => migrator,
            Err(e) => {
                log::error!("migrator error {e}, fall back to relative path");
                sqlx::migrate::Migrator::new(Path::new("./migrations")).await?
            }
        }
        .run(conn)
        .await?;

        let db = SqliteDatabase { conn: conn.clone() };

        Ok(db)
    }

    pub async fn count_accounts_entries(&mut self) -> usize {
        let query: &str = "SELECT COUNT(*) FROM UserAccounts;";
        let result: u64 = sqlx::query_scalar(query)
            .fetch_one(&self.conn)
            .await
            .unwrap();
        result as usize
    }
    pub async fn select_accounts(&mut self) -> Vec<Account> {
        let query: &str = "SELECT * FROM UserAccounts;";
        let result: Vec<Account> = sqlx::query_as(query).fetch_all(&self.conn).await.unwrap();

        result
    }
    pub async fn select_accounts_with_limit(&mut self, limit: u8, offset: usize) -> Vec<Account> {
        let query: &str = "SELECT * FROM UserAccounts LIMIT $1 OFFSET $2;";
        let result = sqlx::query(query)
            .bind(limit)
            .bind(offset.to_string())
            .fetch_all(&self.conn)
            .await
            .unwrap();

        let data: Vec<Account> = result
            .iter()
            .map(|row| Account {
                id: row.get("id"),
                display_name: row.get("display_name"),
                api_token: row.get("api_token"),
                instance: row.get("instance"),
                last_sync_status: row.get("last_sync_status"),
                last_sync_timestamp: row.get("last_sync_timestamp"),
                tls: row.get("tls"),
                enable_sharing: row.get("enable_sharing"),
                enable_public_sharing: row.get("enable_public_sharing"),
            })
            .collect();
        data
    }
    pub async fn delete_account(&mut self, account_id: i64) {
        let bookmarks_query: &str = "DELETE FROM UserAccounts WHERE id = $1;";
        sqlx::query(bookmarks_query)
            .bind(account_id)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn update_account(&mut self, account: &Account) {
        let query: &str = "UPDATE UserAccounts SET display_name=$2, instance=$3, api_token=$4, tls=$5, enable_sharing=$6, enable_public_sharing=$7 WHERE id=$1;";
        sqlx::query(query)
            .bind(account.id)
            .bind(&account.display_name)
            .bind(&account.instance)
            .bind(&account.api_token)
            .bind(account.tls)
            .bind(account.enable_sharing)
            .bind(account.enable_public_sharing)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn create_account(&mut self, account: &Account) {
        let query: &str = "INSERT INTO UserAccounts (display_name, instance, api_token, last_sync_status, last_sync_timestamp, tls, enable_sharing, enable_public_sharing) VALUES ($1, $2, $3, 0, 0, $4, $5, $6);";
        sqlx::query(query)
            .bind(&account.display_name)
            .bind(&account.instance)
            .bind(&account.api_token)
            .bind(account.tls)
            .bind(account.enable_sharing)
            .bind(account.enable_public_sharing)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    //NOTE: (vkhitrin) at the moment, this function is no longer required.
    //                 Perhaps it should be removed/refactored.
    //TODO: (vkhitrin) this is a dumb "cache" that wipes all previous entries.
    //                 It should be improved in the future.
    //pub async fn cache_all_bookmarks(&mut self, bookmarks: &Vec<Bookmark>, epoch_timestamp: i64) {
    //    let truncate_query: &str = "DELETE FROM Bookmarks;";
    //    let update_timestamp_query =
    //        "UPDATE UserAccounts SET last_sync_status=$1, last_sync_timestamp=$2";
    //    sqlx::query(truncate_query)
    //        .execute(&self.conn)
    //        .await
    //        .unwrap();
    //    if !bookmarks.is_empty() {
    //        for bookmark in bookmarks {
    //            self.add_bookmark(&bookmark).await;
    //        }
    //    }
    //    sqlx::query(update_timestamp_query)
    //        .bind(1)
    //        .bind(epoch_timestamp)
    //        .execute(&self.conn)
    //        .await
    //        .unwrap();
    //}
    pub async fn aggregate_bookmarks_for_acount(
        &mut self,
        account: &Account,
        bookmarks: Vec<Bookmark>,
        epoch_timestamp: i64,
        response_successful: bool,
    ) {
        let delete_query: &str = "DELETE FROM Bookmarks where user_account_id = $1;";
        let update_timestamp_query =
            "UPDATE UserAccounts SET last_sync_status=$2, last_sync_timestamp=$3 WHERE id=$1";
        if response_successful {
            sqlx::query(delete_query)
                .bind(account.id)
                .execute(&self.conn)
                .await
                .unwrap();
            for bookmark in bookmarks {
                self.add_bookmark(&bookmark).await;
            }
        }
        sqlx::query(update_timestamp_query)
            .bind(account.id)
            .bind(response_successful)
            .bind(epoch_timestamp)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn add_bookmark(&mut self, bookmark: &Bookmark) {
        let query: &str = r"
            INSERT INTO Bookmarks (
                user_account_id,
                linkding_internal_id,
                url,
                title,
                description,
                notes,
                web_archive_snapshot_url,
                favicon_url,
                preview_image_url,
                is_archived,
                unread,
                shared,
                tag_names,
                date_added,
                date_modified,
                website_title,
                website_description)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17);";
        sqlx::query(query)
            .bind(bookmark.user_account_id)
            .bind(bookmark.linkding_internal_id)
            .bind(&bookmark.url)
            .bind(&bookmark.title)
            .bind(&bookmark.description)
            .bind(&bookmark.notes)
            .bind(&bookmark.web_archive_snapshot_url)
            .bind(&bookmark.favicon_url)
            .bind(&bookmark.preview_image_url)
            .bind(bookmark.is_archived)
            .bind(bookmark.unread)
            .bind(bookmark.shared)
            .bind(bookmark.tag_names.join(" "))
            .bind(&bookmark.date_added)
            .bind(&bookmark.date_modified)
            .bind(&bookmark.website_title)
            .bind(&bookmark.website_description)
            .execute(&self.conn)
            .await
            .unwrap();
    }

    pub async fn count_bookmarks_entries(&mut self) -> usize {
        let query: &str = "SELECT COUNT(*) FROM Bookmarks;";
        let result: u64 = sqlx::query_scalar(query)
            .fetch_one(&self.conn)
            .await
            .unwrap();
        result as usize
    }
    pub async fn select_bookmarks_with_limit(
        &mut self,
        limit: u8,
        offset: usize,
        order_by: SortOption,
    ) -> Vec<Bookmark> {
        let order_by_string = match order_by {
            SortOption::BookmarksDateNewest => "date_added DESC",
            SortOption::BookmarksDateOldest => "date_added ASC",
            SortOption::BookmarkAlphabeticalAscending => "title COLLATE NOCASE ASC",
            SortOption::BookmarkAlphabeticalDescending => "title COLLATE NOCASE DESC",
        };
        let query: String =
            format!("SELECT * FROM Bookmarks ORDER BY {order_by_string} LIMIT $1 OFFSET $2;");

        let result = sqlx::query(&query)
            .bind(limit)
            .bind(offset.to_string())
            .fetch_all(&self.conn)
            .await
            .unwrap();

        let data: Vec<Bookmark> = result
            .iter()
            .map(|row| {
                let tags_string: String = row.get("tag_names");
                let mut tags: Vec<String> = Vec::new();
                if !tags_string.is_empty() {
                    tags = tags_string
                        .split(' ')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                Bookmark {
                    id: row.get("id"),
                    linkding_internal_id: row.get("linkding_internal_id"),
                    user_account_id: row.get("user_account_id"),
                    url: row.get("url"),
                    title: row.get("title"),
                    description: row.get("description"),
                    notes: row.get("notes"),
                    web_archive_snapshot_url: row.get("web_archive_snapshot_url"),
                    favicon_url: row.get("favicon_url"),
                    preview_image_url: row.get("preview_image_url"),
                    is_archived: row.get("is_archived"),
                    unread: row.get("unread"),
                    shared: row.get("shared"),
                    tag_names: tags,
                    date_added: row.get("date_added"),
                    date_modified: row.get("date_modified"),
                    website_title: row.get("website_title"),
                    website_description: row.get("website_description"),
                }
            })
            .collect();
        data
    }
    pub async fn update_bookmark(&mut self, old_bookmark: &Bookmark, new_bookmark: &Bookmark) {
        let query: &str = r"
            UPDATE Bookmarks SET
                url=$1,
                title=$2,
                description=$3,
                notes=$4,
                web_archive_snapshot_url=$5,
                favicon_url=$6,
                preview_image_url=$7,
                is_archived=$8,
                unread=$9,
                shared=$10,
                tag_names=$11,
                date_added=$12,
                date_modified=$13,
                website_title=$14,
                website_description=$15
            WHERE linkding_internal_id=$16;";
        sqlx::query(query)
            .bind(&new_bookmark.url)
            .bind(&new_bookmark.title)
            .bind(&new_bookmark.description)
            .bind(&new_bookmark.notes)
            .bind(&new_bookmark.web_archive_snapshot_url)
            .bind(&new_bookmark.favicon_url)
            .bind(&new_bookmark.preview_image_url)
            .bind(new_bookmark.is_archived)
            .bind(new_bookmark.unread)
            .bind(new_bookmark.shared)
            .bind(new_bookmark.tag_names.join(" "))
            .bind(&new_bookmark.date_added)
            .bind(&new_bookmark.date_modified)
            .bind(&new_bookmark.website_title)
            .bind(&new_bookmark.website_description)
            .bind(old_bookmark.linkding_internal_id)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn delete_all_bookmarks_of_account(&mut self, account_id: i64) {
        let query: &str = "DELETE FROM Bookmarks WHERE user_account_id = $1;";
        sqlx::query(query)
            .bind(account_id)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn delete_bookmark(&mut self, bookmark: &Bookmark) {
        let query: &str = "DELETE FROM Bookmarks WHERE id = $1;";
        sqlx::query(query)
            .bind(bookmark.id)
            .execute(&self.conn)
            .await
            .unwrap();
    }
    pub async fn search_bookmarks(
        &mut self,
        search_string: String,
        limit: u8,
        offset: usize,
        order_by: SortOption,
    ) -> (usize, Vec<Bookmark>) {
        let order_by_string = match order_by {
            SortOption::BookmarksDateNewest => "date_added DESC",
            SortOption::BookmarksDateOldest => "date_added ASC",
            SortOption::BookmarkAlphabeticalAscending => "title COLLATE NOCASE ASC",
            SortOption::BookmarkAlphabeticalDescending => "title COLLATE NOCASE DESC",
        };
        let query = format!(
            r"
        WITH bookmark_count AS (
            SELECT COUNT(*) AS count FROM Bookmarks
            WHERE (
                coalesce(url, '') || ' ' ||
                coalesce(title, '') || ' ' ||
                coalesce(description, '') || ' ' ||
                coalesce(notes, '') || ' ' ||
                coalesce(tag_names, '')
            )
            LIKE '%' || $1 || '%'
        )
        SELECT * FROM Bookmarks, bookmark_count
        WHERE (
            coalesce(url, '') || ' ' ||
            coalesce(title, '') || ' ' ||
            coalesce(description, '') || ' ' ||
            coalesce(notes, '') || ' ' ||
            coalesce(tag_names, '')
        )
        LIKE '%' || $1 || '%'
        ORDER BY {order_by_string}
        LIMIT $2 OFFSET $3;"
        );

        let result = sqlx::query(&query)
            .bind(&search_string)
            .bind(limit)
            .bind(offset.to_string())
            .fetch_all(&self.conn)
            .await
            .unwrap();

        let row_count: usize = result
            .first()
            .map_or(0, |row| row.get::<i64, _>("count") as usize);

        let data: Vec<Bookmark> = result
            .iter()
            .map(|row| {
                let tags_string: String = row.get("tag_names");
                let mut tags: Vec<String> = Vec::new();
                if !tags_string.is_empty() {
                    tags = tags_string
                        .split(' ')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                Bookmark {
                    id: row.get("id"),
                    linkding_internal_id: row.get("linkding_internal_id"),
                    user_account_id: row.get("user_account_id"),
                    url: row.get("url"),
                    title: row.get("title"),
                    description: row.get("description"),
                    notes: row.get("notes"),
                    web_archive_snapshot_url: row.get("web_archive_snapshot_url"),
                    favicon_url: row.get("favicon_url"),
                    preview_image_url: row.get("preview_image_url"),
                    is_archived: row.get("is_archived"),
                    unread: row.get("unread"),
                    shared: row.get("shared"),
                    tag_names: tags,
                    date_added: row.get("date_added"),
                    date_modified: row.get("date_modified"),
                    website_title: row.get("website_title"),
                    website_description: row.get("website_description"),
                }
            })
            .collect();

        (row_count, data)
    }
    pub async fn select_single_account(&mut self, account_id: i64) -> Account {
        let query: &str = "SELECT * FROM UserAccounts WHERE id = $1;";
        let result: Account = sqlx::query_as(query)
            .bind(account_id)
            .fetch_one(&self.conn)
            .await
            .unwrap();
        result
    }
    pub async fn check_if_account_exists(&mut self, url: &String, api_token: &String) -> bool {
        let query: &str =
            "SELECT COUNT(*) FROM UserAccounts WHERE instance = $1 AND api_token = $2;";
        let result: bool = sqlx::query_scalar(query)
            .bind(url)
            .bind(api_token)
            .fetch_one(&self.conn)
            .await
            .unwrap();
        result
    }
}
