use std::collections::HashMap;

use crate::config::SortOption;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{
    widget::menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};

use crate::{
    app::{ApplicationState, MenuAction, Message},
    fl,
};

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
                    Item::Button(fl!("add-account"), MenuAction::AddAccount),
                    if accounts_present && matches!(app_state, ApplicationState::Normal) {
                        Item::Button(fl!("add-bookmark"), MenuAction::AddBookmark)
                    } else {
                        Item::ButtonDisabled(fl!("add-bookmark"), MenuAction::AddBookmark)
                    },
                    Item::Divider,
                    if bookmarks_present && matches!(app_state, ApplicationState::Normal) {
                        Item::Button(fl!("refresh-bookmarks"), MenuAction::RefreshBookmarks)
                    } else {
                        Item::ButtonDisabled(fl!("refresh-bookmarks"), MenuAction::Empty)
                    },
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("view")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("about"), MenuAction::About),
                    Item::Button(fl!("settings"), MenuAction::Settings),
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
                            matches!(sort_option, SortOption::BookmarksDateNewest),
                            MenuAction::SetSortBookmarks(SortOption::BookmarksDateNewest),
                        ),
                        Item::CheckBox(
                            fl!("bookmark-date-oldest"),
                            matches!(sort_option, SortOption::BookmarksDateOldest),
                            MenuAction::SetSortBookmarks(SortOption::BookmarksDateOldest),
                        ),
                        Item::Divider,
                        Item::CheckBox(
                            fl!("bookmark-alphabetical-ascending"),
                            matches!(sort_option, SortOption::BookmarkAlphabeticalAscending),
                            MenuAction::SetSortBookmarks(SortOption::BookmarkAlphabeticalAscending),
                        ),
                        Item::CheckBox(
                            fl!("bookmark-alphabetical-descending"),
                            matches!(sort_option, SortOption::BookmarkAlphabeticalDescending),
                            MenuAction::SetSortBookmarks(
                                SortOption::BookmarkAlphabeticalDescending,
                            ),
                        ),
                    ]
                } else {
                    vec![
                        Item::ButtonDisabled(fl!("bookmark-date-newest"), MenuAction::Empty),
                        Item::ButtonDisabled(fl!("bookmark-date-oldest"), MenuAction::Empty),
                        Item::Divider,
                        Item::ButtonDisabled(
                            fl!("bookmark-alphabetical-ascending"),
                            MenuAction::Empty,
                        ),
                        Item::ButtonDisabled(
                            fl!("bookmark-alphabetical-descending"),
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
