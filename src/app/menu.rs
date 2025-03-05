use std::collections::HashMap;

use crate::app::config::SortOption;
use cosmic::widget::menu::{action::MenuAction as _MenuAction, key_bind::KeyBind};
use cosmic::{
    widget::menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};

use crate::{
    app::{ApplicationState, ContextPage, Message},
    fl,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    AddAccount,
    AddBookmark,
    Empty,
    RefreshBookmarks,
    Settings,
    SetSortBookmarks(SortOption),
}

impl _MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Empty => Message::Empty,
            MenuAction::AddAccount => Message::AddAccount,
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::AddBookmark => Message::AddBookmarkForm,
            MenuAction::RefreshBookmarks => Message::StartRefreshBookmarksForAllAccounts,
            MenuAction::SetSortBookmarks(option) => Message::SortOption(*option),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, MenuAction>,
    accounts_present: bool,
    bookmarks_present: bool,
    sort_option: SortOption,
    app_state: ApplicationState,
) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            root(fl!("file")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("add-account"), None, MenuAction::AddAccount),
                    if accounts_present && matches!(app_state, ApplicationState::Normal) {
                        Item::Button(fl!("add-bookmark"), None, MenuAction::AddBookmark)
                    } else {
                        Item::ButtonDisabled(fl!("add-bookmark"), None, MenuAction::AddBookmark)
                    },
                    Item::Divider,
                    if bookmarks_present && matches!(app_state, ApplicationState::Normal) {
                        Item::Button(fl!("refresh-bookmarks"), None, MenuAction::RefreshBookmarks)
                    } else {
                        Item::ButtonDisabled(fl!("refresh-bookmarks"), None, MenuAction::Empty)
                    },
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("view")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("about"), None, MenuAction::About),
                    Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        ),
        // TODO: (vkhitrin) dynamically generate enabled/disabled entries
        //       instead of writing manual code
        Tree::with_children(
            root(fl!("sort")),
            items(
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
