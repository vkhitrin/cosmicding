use std::collections::HashMap;

use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{
    widget::menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};

use crate::{
    app::{MenuAction, Message},
    fl,
};

pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, MenuAction>,
    accounts_present: bool,
    bookmarks_present: bool,
) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            root(fl!("file")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("add-account"), MenuAction::AddAccount),
                    if accounts_present {
                        Item::Button(fl!("add-bookmark"), MenuAction::AddBookmark)
                    } else {
                        Item::ButtonDisabled(fl!("add-bookmark"), MenuAction::AddBookmark)
                    },
                    Item::Divider,
                    if bookmarks_present {
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
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
