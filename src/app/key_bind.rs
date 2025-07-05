use std::collections::HashMap;

use cosmic::{
    iced::keyboard::Key,
    widget::menu::key_bind::{KeyBind, Modifier},
};

use crate::app::menu::MenuAction;

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$($modifier),*],
                    key: $key,
                },
                MenuAction::$action,
            );
        }};
    }

    let primary_modifier = if cfg!(target_os = "macos") {
        Modifier::Super
    } else {
        Modifier::Ctrl
    };

    let secondary_modifier = Modifier::Shift;

    bind!([primary_modifier], Key::Character(",".into()), Settings);
    bind!(
        [primary_modifier, secondary_modifier],
        Key::Character("n".into()),
        AddAccount
    );
    bind!([primary_modifier], Key::Character("n".into()), AddBookmark);
    bind!(
        [primary_modifier],
        Key::Character("r".into()),
        RefreshBookmarks
    );
    bind!(
        [primary_modifier],
        Key::Character("f".into()),
        SearchActivate
    );

    key_binds
}
