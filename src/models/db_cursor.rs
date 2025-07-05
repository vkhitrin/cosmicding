use crate::app::config::SortOption;
use crate::db::SqliteDatabase;
use crate::models::{account::Account, bookmarks::Bookmark};

pub trait Pagination {
    async fn refresh_count(&mut self);
    async fn refresh_offset(&mut self, page_index: usize);
    async fn fetch_next_results(&mut self);
}

#[derive(Debug, Default, Clone)]
pub struct BookmarksPaginationCursor {
    offset: usize,
    pub search_query: Option<String>,
    pub current_page: usize,
    pub database: Option<SqliteDatabase>,
    pub items_per_page: u8,
    pub sort_option: SortOption,
    pub result: Option<Vec<Bookmark>>,
    pub total_entries: usize,
    pub total_pages: usize,
}

impl BookmarksPaginationCursor {
    pub fn new(database: SqliteDatabase) -> Self {
        Self {
            offset: 0,
            search_query: None,
            current_page: 1,
            database: Some(database),
            items_per_page: 0,
            sort_option: SortOption::BookmarksDateNewest,
            result: None,
            total_entries: 0,
            total_pages: 1,
        }
    }
}

impl Pagination for BookmarksPaginationCursor {
    async fn refresh_count(&mut self) {
        if let Some(database) = &mut self.database {
            if self.search_query.is_none() {
                self.total_entries = database.count_bookmarks_entries().await;
            }
            self.total_pages = std::cmp::max(
                (self.total_entries as f64 / f64::from(self.items_per_page)).ceil() as usize,
                1,
            );
        }
        if self.current_page > self.total_pages {
            self.current_page = self.total_pages;
        }
    }

    async fn refresh_offset(&mut self, page_index: usize) {
        if page_index == 0 {
            self.offset = 0;
            self.current_page = 1;
        } else {
            self.offset = page_index * self.items_per_page as usize;
        }
    }

    async fn fetch_next_results(&mut self) {
        self.refresh_offset(self.current_page - 1).await;
        if let Some(database) = &mut self.database {
            if self.search_query.is_none() {
                self.result = Some(
                    database
                        .select_bookmarks_with_limit(
                            self.items_per_page,
                            self.offset,
                            self.sort_option,
                        )
                        .await,
                );
            } else {
                let (count, bookmarks) = database
                    .search_bookmarks(
                        self.search_query.as_ref().unwrap().to_string(),
                        self.items_per_page,
                        self.offset,
                        self.sort_option,
                    )
                    .await;
                self.total_entries = count;
                self.refresh_count().await;
                if bookmarks.is_empty() {
                    self.result = Some([].to_vec());
                } else {
                    self.result = Some(bookmarks);
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AccountsPaginationCursor {
    offset: usize,
    pub current_page: usize,
    pub database: Option<SqliteDatabase>,
    pub items_per_page: u8,
    pub result: Option<Vec<Account>>,
    pub total_entries: usize,
    pub total_pages: usize,
}

impl AccountsPaginationCursor {
    pub fn new(database: SqliteDatabase) -> Self {
        Self {
            offset: 0,
            current_page: 1,
            database: Some(database),
            items_per_page: 0,
            result: None,
            total_entries: 0,
            total_pages: 1,
        }
    }
}

impl Pagination for AccountsPaginationCursor {
    async fn refresh_count(&mut self) {
        if let Some(database) = &mut self.database {
            self.total_entries = database.count_accounts_entries().await;
            self.total_pages = std::cmp::max(
                (self.total_entries as f64 / f64::from(self.items_per_page)).ceil() as usize,
                1,
            );
        }
        if self.current_page > self.total_pages {
            self.current_page = self.total_pages;
            self.fetch_next_results().await;
        }
    }

    async fn refresh_offset(&mut self, page_index: usize) {
        if page_index == 0 {
            self.offset = 0;
            self.current_page = 1;
        } else {
            self.offset = page_index * self.items_per_page as usize;
        }
    }

    async fn fetch_next_results(&mut self) {
        self.refresh_offset(self.current_page - 1).await;
        if let Some(database) = &mut self.database {
            self.result = Some(
                database
                    .select_accounts_with_limit(self.items_per_page, self.offset)
                    .await,
            );
        }
    }
}
