use std::collections::HashMap;

use cosmic::iced::keyboard::Key;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::key_bind::Modifier;

use crate::app::MenuAction;

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                MenuAction::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character(",".into()), Settings);
    bind!([Ctrl], Key::Character("i".into()), About);
    bind!([Ctrl, Shift], Key::Character("n".into()), AddAccount);
    bind!([Ctrl], Key::Character("n".into()), AddBookmark);

    key_binds
}
