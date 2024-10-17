use anyhow::{anyhow, Result};

use std::path::Path;

use sqlx::{migrate::MigrateDatabase, prelude::*, Sqlite, SqliteConnection};

use crate::app::{APP, APPID, ORG, QUALIFIER};
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;

const DB_PATH: &str = constcat::concat!(APPID, "-db", ".sqlite");

#[derive(Debug)]
pub struct SqliteDatabase {
    conn: SqliteConnection,
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

        let mut conn = SqliteConnection::connect(db_path).await?;

        let migration_path = db_dir.join("migrations");
        std::fs::create_dir_all(&migration_path)?;
        include_dir::include_dir!("migrations")
            .extract(&migration_path)
            .unwrap();

        match sqlx::migrate::Migrator::new(migration_path).await {
            Ok(migrator) => migrator,
            Err(e) => {
                println!("migrator error {e}, fall back to relative path");
                sqlx::migrate::Migrator::new(Path::new("./migrations")).await?
            }
        }
        .run(&mut conn)
        .await?;

        let db = SqliteDatabase { conn };

        Ok(db)
    }

    pub async fn fetch_accounts(&mut self) -> Vec<Account> {
        let query: &str = "SELECT * FROM UserAccounts;";
        let result = sqlx::query(query).fetch_all(&mut self.conn).await.unwrap();

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
            })
            .collect();
        return data;
    }
    pub async fn delete_account(&mut self, account: &Account) {
        let bookmarks_query: &str = "DELETE FROM UserAccounts WHERE id = $1;";
        sqlx::query(bookmarks_query)
            .bind(account.id)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn update_account(&mut self, account: &Account) {
        let query: &str = "UPDATE UserAccounts SET display_name=$2, instance=$3, api_token=$4, tls=$5 WHERE id=$1;";
        sqlx::query(query)
            .bind(&account.id)
            .bind(&account.display_name)
            .bind(&account.instance)
            .bind(&account.api_token)
            .bind(&account.tls)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn create_account(&mut self, account: &Account) {
        let query: &str = "INSERT INTO UserAccounts (display_name, instance, api_token, last_sync_status, last_sync_timestamp, tls) VALUES ($1, $2, $3, 0, 0, $4);";
        sqlx::query(query)
            .bind(&account.display_name)
            .bind(&account.instance)
            .bind(&account.api_token)
            .bind(&account.tls)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    //TODO: (vkhitrin) this is a dumb "cache" that wipes all previous entries.
    //                 It should be improved in the future.
    pub async fn cache_all_bookmarks(&mut self, bookmarks: Vec<Bookmark>) {
        let truncate_query: &str = "DELETE FROM Bookmarks;";
        sqlx::query(truncate_query)
            .execute(&mut self.conn)
            .await
            .unwrap();
        if !bookmarks.is_empty() {
            for bookmark in bookmarks {
                self.add_bookmark(&bookmark).await;
            }
        }
    }
    pub async fn cache_bookmarks_for_acount(
        &mut self,
        account: &Account,
        bookmarks: Vec<Bookmark>,
    ) {
        let truncate_query: &str = "DELETE FROM Bookmarks where user_account_id = $1;";
        sqlx::query(truncate_query)
            .bind(account.id)
            .execute(&mut self.conn)
            .await
            .unwrap();
        if !bookmarks.is_empty() {
            for bookmark in bookmarks {
                self.add_bookmark(&bookmark).await;
            }
        }
    }
    pub async fn add_bookmark(&mut self, bookmark: &Bookmark) {
        let query: &str = r#"
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17);"#;
        sqlx::query(query)
            .bind(&bookmark.user_account_id)
            .bind(&bookmark.linkding_internal_id)
            .bind(&bookmark.url)
            .bind(&bookmark.title)
            .bind(&bookmark.description)
            .bind(&bookmark.notes)
            .bind(&bookmark.web_archive_snapshot_url)
            .bind(&bookmark.favicon_url)
            .bind(&bookmark.preview_image_url)
            .bind(&bookmark.is_archived)
            .bind(&bookmark.unread)
            .bind(&bookmark.shared)
            .bind(&bookmark.tag_names.join(" "))
            .bind(&bookmark.date_added)
            .bind(&bookmark.date_modified)
            .bind(&bookmark.website_title)
            .bind(&bookmark.website_description)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn fetch_bookmarks(&mut self) -> Vec<Bookmark> {
        let query: &str = "SELECT * FROM Bookmarks;";
        let result = sqlx::query(query).fetch_all(&mut self.conn).await.unwrap();

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
        return data;
    }
    pub async fn update_bookmark(&mut self, old_bookmark: &Bookmark, new_bookmark: &Bookmark) {
        let query: &str = r#"
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
            WHERE linkding_internal_id=$16;"#;
        sqlx::query(query)
            .bind(&new_bookmark.url)
            .bind(&new_bookmark.title)
            .bind(&new_bookmark.description)
            .bind(&new_bookmark.notes)
            .bind(&new_bookmark.web_archive_snapshot_url)
            .bind(&new_bookmark.favicon_url)
            .bind(&new_bookmark.preview_image_url)
            .bind(&new_bookmark.is_archived)
            .bind(&new_bookmark.unread)
            .bind(&new_bookmark.shared)
            .bind(&new_bookmark.tag_names.join(" "))
            .bind(&new_bookmark.date_added)
            .bind(&new_bookmark.date_modified)
            .bind(&new_bookmark.website_title)
            .bind(&new_bookmark.website_description)
            .bind(&old_bookmark.id)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn delete_all_bookmarks_of_account(&mut self, account: &Account) {
        let query: &str = "DELETE FROM Bookmarks WHERE user_account_id = $1;";
        sqlx::query(query)
            .bind(&account.id)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn delete_bookmark(&mut self, bookmark: &Bookmark) {
        let query: &str = "DELETE FROM Bookmarks WHERE id = $1;";
        sqlx::query(query)
            .bind(&bookmark.id)
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
    pub async fn search_bookmarks(&mut self, input: String) -> Vec<Bookmark> {
        let query: &str = r#"
            SELECT * FROM Bookmarks 
            WHERE (
                (
                    coalesce(url, '') || ' ' ||
                    coalesce(title, '') || ' ' ||
                    coalesce(description, '') || ' ' ||
                    coalesce(notes, '') || ' ' ||
                    coalesce(tag_names, '')
                )
                LIKE '%' || $1 || '%'
            );"#;
        let result = sqlx::query(query)
            .bind(input)
            .fetch_all(&mut self.conn)
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
        return data;
    }
}
