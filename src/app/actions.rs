use crate::app::{
    config::{AppTheme, CosmicConfig, SortOption},
    context::ContextPage,
    dialog::DialogPage,
};
use crate::models::account::{Account, LinkdingAccountApiResponse};
use crate::models::bookmarks::{Bookmark, DetailedResponse};
use cosmic::widget::{self};
use cosmic::{
    iced::keyboard::{Key, Modifiers},
    iced_core::image::Bytes,
};

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    AccountsView(AccountsAction),
    AddAccount,
    AddBookmark(Account, Bookmark),
    AddBookmarkForm,
    AddBookmarkFormAccountIndex(usize),
    AppTheme(AppTheme),
    BookmarksView(BookmarksAction),
    CloseToast(widget::ToastId),
    CompleteAddAccount(Account),
    CompleteRemoveDialog(Option<i64>, Option<Bookmark>),
    ContextClose,
    DecrementPageIndex(String),
    DialogCancel,
    DialogUpdate(DialogPage),
    DoneFetchFaviconForBookmark(String, Bytes),
    DoneRefreshAccountProfile(Account, Option<LinkdingAccountApiResponse>),
    DoneRefreshBookmarksForAccount(Account, Vec<DetailedResponse>),
    DoneRefreshBookmarksForAllAccounts(Vec<DetailedResponse>),
    EditAccount(Account),
    EditBookmark(i64, Bookmark),
    Empty,
    EnableFavicons(bool),
    IncrementPageIndex(String),
    InputBookmarkDescription(widget::text_editor::Action),
    InputBookmarkNotes(widget::text_editor::Action),
    Key(Modifiers, Key),
    LoadAccounts,
    LoadBookmarks,
    Modifiers(Modifiers),
    OpenAccountsPage,
    OpenExternalUrl(String),
    OpenPurgeFaviconsCache,
    OpenRemoveAccountDialog(Account),
    OpenRemoveBookmarkDialog(i64, Bookmark),
    PurgeFaviconsCache,
    RemoveAccount(Account),
    RemoveBookmark(i64, Bookmark),
    SearchBookmarks(String),
    SetAccountAPIKey(String),
    SetAccountDisplayName(String),
    SetAccountInstance(String),
    SetAccountStatus(bool),
    SetAccountTrustInvalidCertificates(bool),
    SetBookmarkArchived(bool),
    SetBookmarkShared(bool),
    SetBookmarkTags(String),
    SetBookmarkTitle(String),
    SetBookmarkURL(String),
    SetBookmarkUnread(bool),
    SetItemsPerPage(u8),
    SortOption(SortOption),
    StartFetchFaviconForBookmark(Bookmark),
    StartRefreshAccountProfile(Account),
    StartRefreshBookmarksForAccount(Account),
    StartRefreshBookmarksForAllAccounts,
    StartupCompleted,
    SystemThemeModeChange,
    ToggleContextPage(ContextPage),
    UpdateAccount(Account),
    UpdateBookmark(Account, Bookmark),
    UpdateConfig(CosmicConfig),
    ViewBookmarkNotes(Bookmark),
}

#[derive(Debug, Clone)]
pub enum AccountsAction {
    AddAccount,
    DecrementPageIndex,
    DeleteAccount(Account),
    EditAccount(Account),
    IncrementPageIndex,
    OpenExternalURL(String),
    RefreshBookmarksForAccount(Account),
}

#[derive(Debug, Clone)]
pub enum BookmarksAction {
    AddBookmark,
    ClearSearch,
    DecrementPageIndex,
    DeleteBookmark(i64, Bookmark),
    EditBookmark(i64, Bookmark),
    EmptyMessage,
    IncrementPageIndex,
    OpenAccountsPage,
    OpenExternalURL(String),
    RefreshBookmarks,
    SearchBookmarks(String),
    ViewNotes(Bookmark),
}
