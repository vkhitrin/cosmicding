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
            Self::BookmarksView => icon::from_name("web-browser-symbolic").icon(),
            Self::AccountsView => icon::from_name("contact-new-symbolic").icon(),
        }
    }

    pub fn view(self, app: &app::Cosmicding) -> Element<'_, app::Message> {
        match self {
            AppNavPage::AccountsView => app.accounts_view.view().map(app::Message::AccountsView),
            AppNavPage::BookmarksView => app.bookmarks_view.view().map(app::Message::BookmarksView),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::AccountsView, Self::BookmarksView]
    }
}
