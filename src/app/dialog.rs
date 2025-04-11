use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum DialogPage {
    RemoveAccount(Account),
    RemoveBookmark(i64, Bookmark),
    PurgeFaviconsCache(),
}
