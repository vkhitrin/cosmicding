use cosmic::{widget::icon, Element};

use crate::{app, fl};

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum NavPage {
    #[default]
    BookmarksView,
    AccountsView,
}

impl Default for &NavPage {
    fn default() -> Self {
        &NavPage::BookmarksView
    }
}

impl NavPage {
    pub fn title(&self) -> String {
        match self {
            Self::BookmarksView => fl!("bookmarks-view-title"),
            Self::AccountsView => fl!("accounts-nav-view-title"),
        }
    }
    pub fn icon(&self) -> cosmic::widget::Icon {
        match self {
            Self::BookmarksView => icon::from_name("web-browser-symbolic").icon(),
            Self::AccountsView => icon::from_name("contact-new-symbolic").icon(),
        }
    }

    pub fn view<'a>(&self, app: &'a app::AppModel) -> Element<'a, app::Message> {
        match self {
            NavPage::AccountsView => app.accounts_view.view().map(app::Message::AccountsView),
            NavPage::BookmarksView => app.bookmarks_view.view().map(app::Message::BookmarksView),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::AccountsView, Self::BookmarksView]
    }
}
