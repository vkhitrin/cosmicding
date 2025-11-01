use crate::models::{account::Account, bookmarks::Bookmark};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum DialogPage {
    RemoveAccount(Account),
    RemoveBookmark(i64, Bookmark),
    PurgeFaviconsCache(),
    ExportBookmarks(Vec<Account>, Vec<bool>, Option<PathBuf>),
    ImportBookmarks(Vec<Account>, usize, Option<PathBuf>),
}
