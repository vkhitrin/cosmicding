use std::collections::HashMap;

use crate::app::config::SortOption;
use cosmic::{
    widget::{
        menu::{
            self, action::MenuAction as _MenuAction, key_bind::KeyBind, Item, ItemHeight, ItemWidth,
        },
        RcElementWrapper,
    },
    Element,
};

use crate::{
    app::{actions::ApplicationAction, context::ContextPage, ApplicationState},
    fl,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    AddAccount,
    AddBookmark,
    Empty,
    ExportBookmarks,
    ImportBookmarks,
    RefreshBookmarks,
    SearchActivate,
    SetSortBookmarks(SortOption),
    Settings,
}

impl _MenuAction for MenuAction {
    type Message = ApplicationAction;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => ApplicationAction::ToggleContextPage(ContextPage::About),
            MenuAction::AddAccount => ApplicationAction::AddAccountForm,
            MenuAction::AddBookmark => ApplicationAction::AddBookmarkForm,
            MenuAction::Empty => ApplicationAction::Empty,
            MenuAction::ExportBookmarks => ApplicationAction::StartExportBookmarks,
            MenuAction::ImportBookmarks => ApplicationAction::StartImportBookmarks,
            MenuAction::RefreshBookmarks => ApplicationAction::StartRefreshBookmarksForAllAccounts,
            // NOTE: (vkhitrin) this is a workaround for the time being, it shouldn't be a
            //                  'MenuAction'.
            MenuAction::SearchActivate => ApplicationAction::SearchActivate,
            MenuAction::SetSortBookmarks(option) => ApplicationAction::SortOption(*option),
            MenuAction::Settings => ApplicationAction::ToggleContextPage(ContextPage::Settings),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, MenuAction>,
    bookmarks_present: bool,
    sort_option: SortOption,
    app_state: ApplicationState,
) -> Element<'a, ApplicationAction> {
    menu::bar(vec![
        menu::Tree::with_children(
            RcElementWrapper::new(Element::from(menu::root(fl!("file")))),
            menu::items(
                key_binds,
                vec![
                    if matches!(
                        app_state,
                        ApplicationState::Ready | ApplicationState::NoEnabledRemoteAccounts
                    ) {
                        Item::Button(fl!("add-account"), None, MenuAction::AddAccount)
                    } else {
                        Item::ButtonDisabled(fl!("add-account"), None, MenuAction::AddAccount)
                    },
                    if matches!(
                        app_state,
                        ApplicationState::Ready | ApplicationState::NoEnabledRemoteAccounts
                    ) {
                        Item::Button(fl!("add-bookmark"), None, MenuAction::AddBookmark)
                    } else {
                        Item::ButtonDisabled(fl!("add-bookmark"), None, MenuAction::AddBookmark)
                    },
                    Item::Divider,
                    if bookmarks_present && matches!(app_state, ApplicationState::Ready) {
                        Item::Button(fl!("export-bookmarks"), None, MenuAction::ExportBookmarks)
                    } else {
                        Item::ButtonDisabled(fl!("export-bookmarks"), None, MenuAction::Empty)
                    },
                    if matches!(app_state, ApplicationState::Ready) {
                        Item::Button(fl!("import-bookmarks"), None, MenuAction::ImportBookmarks)
                    } else {
                        Item::ButtonDisabled(fl!("import-bookmarks"), None, MenuAction::Empty)
                    },
                    Item::Divider,
                    if bookmarks_present && matches!(app_state, ApplicationState::Ready) {
                        Item::Button(fl!("refresh-bookmarks"), None, MenuAction::RefreshBookmarks)
                    } else {
                        Item::ButtonDisabled(fl!("refresh-bookmarks"), None, MenuAction::Empty)
                    },
                ],
            ),
        ),
        menu::Tree::with_children(
            RcElementWrapper::new(Element::from(menu::root(fl!("view")))),
            menu::items(
                key_binds,
                vec![
                    Item::Button(fl!("about"), None, MenuAction::About),
                    Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        ),
        menu::Tree::with_children(
            RcElementWrapper::new(Element::from(menu::root(fl!("sort")))),
            menu::items(
                key_binds,
                if bookmarks_present {
                    vec![
                        Item::CheckBox(
                            fl!("bookmark-date-newest"),
                            None,
                            matches!(sort_option, SortOption::BookmarksDateNewest),
                            MenuAction::SetSortBookmarks(SortOption::BookmarksDateNewest),
                        ),
                        Item::CheckBox(
                            fl!("bookmark-date-oldest"),
                            None,
                            matches!(sort_option, SortOption::BookmarksDateOldest),
                            MenuAction::SetSortBookmarks(SortOption::BookmarksDateOldest),
                        ),
                        Item::Divider,
                        Item::CheckBox(
                            fl!("bookmark-alphabetical-ascending"),
                            None,
                            matches!(sort_option, SortOption::BookmarkAlphabeticalAscending),
                            MenuAction::SetSortBookmarks(SortOption::BookmarkAlphabeticalAscending),
                        ),
                        Item::CheckBox(
                            fl!("bookmark-alphabetical-descending"),
                            None,
                            matches!(sort_option, SortOption::BookmarkAlphabeticalDescending),
                            MenuAction::SetSortBookmarks(
                                SortOption::BookmarkAlphabeticalDescending,
                            ),
                        ),
                    ]
                } else {
                    vec![
                        Item::ButtonDisabled(fl!("bookmark-date-newest"), None, MenuAction::Empty),
                        Item::ButtonDisabled(fl!("bookmark-date-oldest"), None, MenuAction::Empty),
                        Item::Divider,
                        Item::ButtonDisabled(
                            fl!("bookmark-alphabetical-ascending"),
                            None,
                            MenuAction::Empty,
                        ),
                        Item::ButtonDisabled(
                            fl!("bookmark-alphabetical-descending"),
                            None,
                            MenuAction::Empty,
                        ),
                    ]
                },
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
