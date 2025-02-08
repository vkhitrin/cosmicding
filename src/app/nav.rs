use crate::app::icons::load_icon;
use cosmic::{widget::icon, Element};

use crate::{app, fl};

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum AppNavPage {
    #[default]
    BookmarksView,
    AccountsView,
}

impl Default for &AppNavPage {
    fn default() -> Self {
        &AppNavPage::BookmarksView
    }
}

impl AppNavPage {
    pub fn title(self) -> String {
        match self {
            Self::BookmarksView => fl!("bookmarks"),
            Self::AccountsView => fl!("accounts"),
        }
    }
    pub fn icon(self) -> cosmic::widget::Icon {
        match self {
            Self::BookmarksView => icon::icon(load_icon("web-browser-symbolic")),
            Self::AccountsView => icon::icon(load_icon("contact-new-symbolic")),
        }
    }

    pub fn view(self, app: &app::Cosmicding) -> Element<'_, app::Message> {
        match self {
            AppNavPage::AccountsView => app
                .accounts_view
                .view(app.state, &app.accounts_cursor)
                .map(app::Message::AccountsView),
            AppNavPage::BookmarksView => app
                .bookmarks_view
                .view(app.state, &app.bookmarks_cursor)
                .map(app::Message::BookmarksView),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::AccountsView, Self::BookmarksView]
    }
}
