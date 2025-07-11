use cosmic::{widget::icon, Element};

use crate::{
    app::{actions::ApplicationAction, Cosmicding},
    fl,
};

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
            Self::BookmarksView => icon::from_name("web-browser-symbolic").into(),
            Self::AccountsView => icon::from_name("contact-new-symbolic").into(),
        }
    }

    pub fn view(self, app: &Cosmicding) -> Element<'_, ApplicationAction> {
        match self {
            AppNavPage::AccountsView => app
                .accounts_view
                .view(
                    app.state,
                    app.sync_status,
                    &app.accounts_cursor,
                    &app.timeline,
                )
                .map(ApplicationAction::AccountsView),
            AppNavPage::BookmarksView => app
                .bookmarks_view
                .view(
                    app.state,
                    app.sync_status,
                    &app.bookmarks_cursor,
                    app.accounts_cursor.total_entries == 0,
                    &app.timeline,
                )
                .map(ApplicationAction::BookmarksView),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::AccountsView, Self::BookmarksView]
    }
}
