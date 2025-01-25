use crate::config::{AppTheme, Config, SortOption, CONFIG_VERSION};
use crate::db::{self};
use crate::fl;
use crate::http::{self};
use crate::key_binds::key_binds;
use crate::models::account::{Account, LinkdingAccountApiResponse};
use crate::models::bookmarks::{Bookmark, DetailedResponse};
use crate::models::db_cursor::{AccountsPaginationCursor, BookmarksPaginationCursor, Pagination};
use crate::nav::AppNavPage;
use crate::pages::accounts::{add_account, edit_account, AppAccountsMessage, PageAccountsView};
use crate::pages::bookmarks::{
    edit_bookmark, new_bookmark, view_notes, AppBookmarksMessage, PageBookmarksView,
};
use crate::utils::icons::load_icon;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry, Update};
use cosmic::cosmic_theme::{self, ThemeMode};
use cosmic::iced::{
    event,
    futures::executor::block_on,
    keyboard::{Event as KeyEvent, Key, Modifiers},
    Alignment, Event, Length, Subscription,
};
use cosmic::widget::menu::action::MenuAction as _MenuAction;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{theme, Application, ApplicationExt, Element};
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

pub const QUALIFIER: &str = "com";
pub const ORG: &str = "vkhitrin";
pub const APP: &str = "cosmicding";
pub const APPID: &str = constcat::concat!(QUALIFIER, ".", ORG, ".", APP);

const REPOSITORY: &str = "https://github.com/vkhitrin/cosmicding";

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

pub struct Cosmicding {
    core: Core,
    context_page: ContextPage,
    nav: nav_bar::Model,
    dialog_pages: VecDeque<DialogPage>,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    pub config: Config,
    config_handler: Option<cosmic_config::Config>,
    modifiers: Modifiers,
    app_themes: Vec<String>,
    pub bookmarks_cursor: BookmarksPaginationCursor,
    pub accounts_cursor: AccountsPaginationCursor,
    pub accounts_view: PageAccountsView,
    pub bookmarks_view: PageBookmarksView,
    placeholder_account: Option<Account>,
    placeholder_bookmark: Option<Bookmark>,
    placeholder_accounts_list: Vec<Account>,
    placeholder_selected_account_index: usize,
    toasts: widget::toaster::Toasts<Message>,
    pub state: ApplicationState,
}

#[derive(Debug, Clone)]
pub enum Message {
    AccountsView(AppAccountsMessage),
    AddAccount,
    AddBookmark(Account, Bookmark),
    AddBookmarkForm,
    AddBookmarkFormAccountIndex(usize),
    AppTheme(AppTheme),
    BookmarksView(AppBookmarksMessage),
    CloseToast(widget::ToastId),
    CompleteAddAccount(Account),
    CompleteRemoveDialog(i64, Option<Bookmark>),
    DecrementPageIndex(String),
    DialogCancel,
    DialogUpdate(DialogPage),
    DoneRefreshAccountProfile(Account, Option<LinkdingAccountApiResponse>),
    DoneRefreshBookmarksForAccount(Account, Vec<DetailedResponse>),
    DoneRefreshBookmarksForAllAccounts(Vec<DetailedResponse>),
    EditAccount(Account),
    EditBookmark(i64, Bookmark),
    Empty,
    IncrementPageIndex(String),
    Key(Modifiers, Key),
    LoadAccounts,
    LoadBookmarks,
    Modifiers(Modifiers),
    OpenAccountsPage,
    OpenExternalUrl(String),
    OpenRemoveAccountDialog(i64),
    OpenRemoveBookmarkDialog(i64, Bookmark),
    RemoveAccount(i64),
    RemoveBookmark(i64, Bookmark),
    SearchBookmarks(String),
    SetAccountAPIKey(String),
    SetAccountDisplayName(String),
    SetAccountInstance(String),
    SetAccountTLS(bool),
    SetBookmarkArchived(bool),
    SetBookmarkDescription(String),
    SetBookmarkNotes(String),
    SetBookmarkShared(bool),
    SetBookmarkTags(String),
    SetBookmarkTitle(String),
    SetBookmarkURL(String),
    SetBookmarkUnread(bool),
    SetItemsPerPage(u8),
    SortOption(SortOption),
    StartRefreshAccountProfile(Account),
    StartRefreshBookmarksForAccount(Account),
    StartRefreshBookmarksForAllAccounts,
    StartupCompleted,
    SystemThemeModeChange,
    ToggleContextPage(ContextPage),
    UpdateAccount(Account),
    UpdateBookmark(Account, Bookmark),
    UpdateConfig(Config),
    ViewBookmarkNotes(Bookmark),
}

// NOTE: (vkhitrin) look at 'large_enum_variant'
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum DialogPage {
    RemoveAccount(i64),
    RemoveBookmark(i64, Bookmark),
}

#[derive(Debug, Clone, Copy)]
pub enum ApplicationState {
    Normal,
    Refreshing,
    Startup,
}

impl Application for Cosmicding {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = APPID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let db_pool = Some(block_on(async {
            db::SqliteDatabase::create().await.unwrap()
        }));
        let accounts_cursor = AccountsPaginationCursor::new(db_pool.clone().unwrap());
        let bookmarks_cursor = BookmarksPaginationCursor::new(db_pool.clone().unwrap());
        let mut nav = nav_bar::Model::default();
        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        for &nav_page in AppNavPage::all() {
            let id = nav
                .insert()
                .icon(nav_page.icon())
                .text(nav_page.title())
                .data::<AppNavPage>(nav_page)
                .id();

            if nav_page == AppNavPage::default() {
                nav.activate(id);
            }
        }

        let mut app = Cosmicding {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: key_binds(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            config_handler: flags.config_handler,
            modifiers: Modifiers::empty(),
            app_themes,
            bookmarks_cursor,
            accounts_cursor,
            dialog_pages: VecDeque::new(),
            accounts_view: PageAccountsView::default(),
            bookmarks_view: PageBookmarksView::default(),
            placeholder_account: None,
            placeholder_bookmark: None,
            placeholder_accounts_list: Vec::new(),
            placeholder_selected_account_index: 0,
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
            state: ApplicationState::Startup,
        };

        app.bookmarks_cursor.items_per_page = app.config.items_per_page;
        app.accounts_cursor.items_per_page = app.config.items_per_page;

        let commands = vec![
            app.update_title(),
            app.update(Message::SetItemsPerPage(app.config.items_per_page)),
            app.update(Message::StartupCompleted),
        ];

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            app.bookmarks_cursor.refresh_count().await;
        });

        (app, Task::batch(commands))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![crate::menu::menu_bar(
            &self.key_binds,
            !self.accounts_view.accounts.is_empty(),
            !self.bookmarks_view.bookmarks.is_empty(),
            self.config.sort_option,
            self.state,
        )]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_escape(&mut self) -> Task<Message> {
        if self.dialog_pages.pop_front().is_some() {
            return Task::none();
        }

        self.core.window.show_context = false;

        Task::none()
    }

    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::AddAccountForm => add_account(self.placeholder_account.clone().unwrap()),
            ContextPage::EditAccountForm => edit_account(self.placeholder_account.clone().unwrap()),
            ContextPage::NewBookmarkForm => new_bookmark(
                self.placeholder_bookmark.clone().unwrap(),
                &self.placeholder_accounts_list,
                self.placeholder_selected_account_index,
            ),

            ContextPage::EditBookmarkForm => edit_bookmark(
                self.placeholder_bookmark.clone().unwrap(),
                self.placeholder_account.as_ref().unwrap(),
            ),
            ContextPage::ViewBookmarkNotes => {
                view_notes(self.placeholder_bookmark.clone().unwrap())
            }
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::RemoveAccount(account_id) => {
                widget::dialog(fl!("remove") + " " + { &account_id.to_string() })
                    .icon(icon::icon(load_icon("dialog-warning-symbolic")).size(58))
                    .body(fl!("remove-account-confirm"))
                    .primary_action(
                        widget::button::destructive(fl!("yes"))
                            .on_press_maybe(Some(Message::CompleteRemoveDialog(*account_id, None))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
            DialogPage::RemoveBookmark(account, bookmark) => {
                widget::dialog(fl!("remove") + " " + { &bookmark.title })
                    .icon(icon::icon(load_icon("dialog-warning-symbolic")).size(58))
                    .body(fl!("remove-bookmark-confirm"))
                    .primary_action(widget::button::destructive(fl!("yes")).on_press_maybe(Some(
                        Message::CompleteRemoveDialog(*account, Some(bookmark.clone())),
                    )))
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
        };

        Some(dialog.into())
    }

    fn view(&self) -> Element<Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let entity = self.nav.active();
        let nav_page = self.nav.data::<AppNavPage>(entity).unwrap_or_default();

        widget::column::with_children(vec![
            widget::toaster(&self.toasts, widget::horizontal_space()),
            nav_page.view(self),
        ])
        .padding([
            spacing.space_none,
            spacing.space_xs,
            spacing.space_none,
            spacing.space_xs,
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status, _| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    log::info!(
                        "Errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    log::info!(
                        "Errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange
            }),
        ];

        Subscription::batch(subscriptions)
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut commands = vec![];
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        if let Err(err) =
                            paste::paste! { self.config.[<set_ $name>](config_handler, $value) }
                        {
                            log::warn!("Failed to save config {:?}: {}", stringify!($name), err);
                        }
                    }
                    None => {
                        self.config.$name = $value;
                    }
                }
            };
        }
        match message {
            #![allow(non_snake_case)]
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::SortOption(sort_option) => {
                config_set!(sort_option, sort_option);
                if !self.bookmarks_view.bookmarks.is_empty() {
                    return self.update(Message::LoadBookmarks);
                }
            }
            Message::SetItemsPerPage(items_per_page) => {
                config_set!(items_per_page, items_per_page);
                self.bookmarks_cursor.items_per_page = items_per_page;
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    self.accounts_cursor.refresh_count().await;
                    self.accounts_cursor.refresh_offset(0).await;
                    self.bookmarks_cursor.refresh_count().await;
                    self.bookmarks_cursor.refresh_offset(0).await;
                });
                commands.push(self.update(Message::LoadAccounts));
                commands.push(self.update(Message::LoadBookmarks));
            }
            Message::SystemThemeModeChange => {
                return self.update_config();
            }
            Message::OpenAccountsPage => {
                let account_page_entity = &self.nav.entity_at(0);
                self.nav.activate(account_page_entity.unwrap());
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                self.set_context_title(context_page.title());
            }
            Message::AccountsView(message) => commands.push(self.accounts_view.update(message)),
            Message::LoadAccounts => {
                block_on(async {
                    self.accounts_cursor.fetch_next_results().await;
                    self.accounts_cursor.refresh_count().await;
                });
                self.accounts_view.accounts = self.accounts_cursor.result.clone().unwrap();
                // FIXME: (vkhitrin) If an account is deleted during refresh (should not be
                //        possible without interacting with the database manually, a crash will
                //        occur if an account context window is open.
                if self.placeholder_account.is_some() && !self.accounts_view.accounts.is_empty() {
                    self.placeholder_account = self
                        .accounts_view
                        .accounts
                        .clone()
                        .iter()
                        .find(|account| account.id == self.placeholder_account.as_ref().unwrap().id)
                        .cloned();
                }
            }
            Message::AddAccount => {
                self.placeholder_account =
                    Some(Account::new(String::new(), String::new(), String::new()));
                commands.push(self.update(Message::ToggleContextPage(ContextPage::AddAccountForm)));
            }
            Message::EditAccount(account) => {
                self.placeholder_account = Some(account.clone());
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::EditAccountForm)));
            }
            Message::RemoveAccount(account_id) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        db::SqliteDatabase::delete_all_bookmarks_of_account(database, account_id)
                            .await;
                    });
                    block_on(async {
                        db::SqliteDatabase::delete_account(database, account_id).await;
                    });
                    self.bookmarks_view
                        .bookmarks
                        .retain(|bkmrk| bkmrk.user_account_id != Some(account_id));
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "removed-account",
                                acc = account_id
                            )))
                            .map(cosmic::app::Message::App),
                    );
                    block_on(async {
                        self.accounts_cursor.refresh_count().await;
                        self.accounts_cursor.fetch_next_results().await;
                    });
                    self.accounts_view.accounts = self.accounts_cursor.result.clone().unwrap();
                }
            }
            Message::CompleteAddAccount(mut account) => {
                let mut valid_account = false;
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        match http::check_account_on_instance(&account).await {
                            Ok(value) => {
                                account.enable_sharing = value.enable_sharing;
                                account.enable_public_sharing = value.enable_public_sharing;
                                valid_account = true;
                            }
                            Err(e) => {
                                if e.to_string().contains("builder error") {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "provided-url-is-not-valid"
                                            )))
                                            .map(cosmic::app::Message::App),
                                    );
                                } else {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(format!("{e}")))
                                            .map(cosmic::app::Message::App),
                                    );
                                }
                            }
                        }
                    });
                    if valid_account {
                        block_on(async {
                            db::SqliteDatabase::create_account(database, &account).await;
                        });
                        commands.push(self.update(Message::LoadAccounts));
                        commands.push(self.update(Message::StartRefreshBookmarksForAccount(
                            self.accounts_view.accounts.last().unwrap().clone(),
                        )));
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "added-account",
                                    acc = account.display_name
                                )))
                                .map(cosmic::app::Message::App),
                        );
                    }
                }
                self.core.window.show_context = false;
            }
            Message::UpdateAccount(mut account) => {
                let mut valid_account = false;
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        match http::check_account_on_instance(&account).await {
                            Ok(value) => {
                                account.enable_sharing = value.enable_sharing;
                                account.enable_public_sharing = value.enable_public_sharing;
                                valid_account = true;
                            }
                            Err(e) => {
                                if e.to_string().contains("builder error") {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "provided-url-is-not-valid"
                                            )))
                                            .map(cosmic::app::Message::App),
                                    );
                                } else {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(format!("{e}")))
                                            .map(cosmic::app::Message::App),
                                    );
                                }
                            }
                        }
                    });
                    if valid_account {
                        block_on(async {
                            db::SqliteDatabase::update_account(database, &account).await;
                        });
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "updated-account",
                                    acc = account.display_name
                                )))
                                .map(cosmic::app::Message::App),
                        );
                        commands.push(self.update(Message::LoadAccounts));
                        commands.push(self.update(Message::StartRefreshBookmarksForAccount(
                            self.accounts_view.accounts.last().unwrap().clone(),
                        )));
                    }
                }
                self.core.window.show_context = false;
                commands.push(self.update(Message::LoadAccounts));
                self.core.window.show_context = false;
            }
            Message::BookmarksView(message) => commands.push(self.bookmarks_view.update(message)),
            Message::StartRefreshBookmarksForAllAccounts => {
                if let ApplicationState::Refreshing = self.state {
                } else {
                    self.state = ApplicationState::Refreshing;
                    let message = |x: Vec<DetailedResponse>| {
                        cosmic::app::Message::App(Message::DoneRefreshBookmarksForAllAccounts(x))
                    };
                    if !self.accounts_view.accounts.is_empty() {
                        commands.push(Task::perform(
                            http::fetch_bookmarks_from_all_accounts(
                                self.accounts_view.accounts.clone(),
                            ),
                            message,
                        ));
                    }
                }
            }
            Message::DoneRefreshBookmarksForAllAccounts(remote_responses) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    for response in remote_responses {
                        block_on(async {
                            db::SqliteDatabase::aggregate_bookmarks_for_acount(
                                database,
                                &response.account,
                                response.bookmarks.unwrap_or_else(Vec::new),
                                response.timestamp,
                                response.successful,
                            )
                            .await;
                        });
                    }
                    commands.push(self.update(Message::LoadAccounts));
                    commands.push(self.update(Message::LoadBookmarks));
                    self.state = ApplicationState::Normal;
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("refreshed-bookmarks")))
                            .map(cosmic::app::Message::App),
                    );
                }
            }
            Message::StartRefreshBookmarksForAccount(account) => {
                if let ApplicationState::Refreshing = self.state {
                } else {
                    self.state = ApplicationState::Refreshing;
                    let mut acc_vec = self.accounts_view.accounts.clone();
                    acc_vec.retain(|acc| acc.id == account.id);
                    let borrowed_acc = acc_vec[0].clone();
                    let message = move |bookmarks: Vec<DetailedResponse>| {
                        cosmic::app::Message::App(Message::DoneRefreshBookmarksForAccount(
                            borrowed_acc.clone(),
                            bookmarks,
                        ))
                    };
                    commands
                        .push(self.update(Message::StartRefreshAccountProfile(account.clone())));
                    if !acc_vec.is_empty() {
                        commands.push(Task::perform(
                            http::fetch_bookmarks_from_all_accounts(acc_vec.clone()),
                            message,
                        ));
                    }
                }
            }
            Message::DoneRefreshBookmarksForAccount(account, remote_responses) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    for response in remote_responses {
                        block_on(async {
                            db::SqliteDatabase::aggregate_bookmarks_for_acount(
                                database,
                                &account,
                                response.bookmarks.unwrap_or_else(Vec::new),
                                response.timestamp,
                                response.successful,
                            )
                            .await;
                        });
                    }
                    commands.push(self.update(Message::LoadAccounts));
                    commands.push(self.update(Message::LoadBookmarks));
                    self.state = ApplicationState::Normal;
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "refreshed-bookmarks-for-account",
                                acc = account.display_name
                            )))
                            .map(cosmic::app::Message::App),
                    );
                }
            }
            Message::StartRefreshAccountProfile(account) => {
                if let ApplicationState::Refreshing = self.state {
                } else {
                    self.state = ApplicationState::Refreshing;
                    let borrowed_acc = account.clone();
                    let message = move |api_response: Option<LinkdingAccountApiResponse>| {
                        cosmic::app::Message::App(Message::DoneRefreshAccountProfile(
                            borrowed_acc.clone(),
                            api_response,
                        ))
                    };
                    commands.push(Task::perform(http::fetch_account_details(account), message));
                }
            }
            Message::DoneRefreshAccountProfile(mut account, account_details) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if account_details.is_some() {
                        let details = account_details.unwrap();
                        account.enable_sharing = details.enable_sharing;
                        account.enable_public_sharing = details.enable_public_sharing;
                        block_on(async {
                            db::SqliteDatabase::update_account(database, &account).await;
                        });
                        commands.push(self.update(Message::LoadAccounts));
                    } else {
                        block_on(async {
                            db::SqliteDatabase::update_account(database, &account).await;
                        });
                    }
                    self.state = ApplicationState::Normal;
                }
            }
            Message::AddBookmarkForm => {
                // FIXME: (vkhitrin) this should not exist, "bypass" pagination to retrieve all
                //        entries from database. Need to find a better approach to generate a list
                //        of all accounts.
                self.placeholder_accounts_list =
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        db::SqliteDatabase::select_accounts(
                            self.bookmarks_cursor.database.as_mut().unwrap(),
                        )
                        .await
                    });
                if !self.accounts_view.accounts.is_empty() {
                    self.placeholder_bookmark = Some(Bookmark::new(
                        None,
                        None,
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        false,
                        false,
                        false,
                        Vec::new(),
                        None,
                        None,
                    ));
                    commands.push(
                        self.update(Message::ToggleContextPage(ContextPage::NewBookmarkForm)),
                    );
                }
            }
            Message::SetAccountDisplayName(name) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.display_name = name;
                }
            }
            Message::SetAccountInstance(url) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.instance = url;
                }
            }
            Message::SetAccountAPIKey(key) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.api_token = key;
                }
            }
            Message::SetAccountTLS(tls) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.tls = tls;
                }
            }
            Message::AddBookmarkFormAccountIndex(idx) => {
                self.placeholder_selected_account_index = idx;
            }
            Message::SetBookmarkURL(url) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.url = url;
                }
            }
            Message::SetBookmarkTitle(title) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.title = title;
                }
            }
            Message::SetBookmarkDescription(description) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.description = description;
                }
            }
            Message::SetBookmarkNotes(notes) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.notes = notes;
                }
            }
            Message::SetBookmarkTags(tags_string) => {
                let tags: Vec<String> = tags_string
                    .split(' ')
                    .map(|s| s.trim().to_string())
                    .collect();
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.tag_names = tags;
                }
            }
            Message::SetBookmarkArchived(archived) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.is_archived = archived;
                }
            }
            Message::SetBookmarkUnread(unread) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.unread = unread;
                }
            }
            Message::SetBookmarkShared(shared) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.shared = shared;
                }
            }
            Message::AddBookmark(account, bookmark) => {
                let mut new_bkmrk: Option<Bookmark> = None;
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        match http::add_bookmark(&account, &bookmark).await {
                            Ok(value) => {
                                new_bkmrk = Some(value);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "added-bookmark-to-account",
                                            bkmrk = bookmark.url.clone(),
                                            acc = account.display_name.clone()
                                        )))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                            Err(e) => {
                                log::error!("Error adding bookmark: {}", e);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(format!("{e}")))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                        }
                    });
                    if let Some(bkmrk) = new_bkmrk {
                        block_on(async {
                            db::SqliteDatabase::add_bookmark(database, &bkmrk).await;
                        });
                        commands.push(self.update(Message::LoadBookmarks));
                    }
                };
                block_on(async {
                    self.bookmarks_cursor.refresh_count().await;
                });
                self.core.window.show_context = false;
            }
            Message::RemoveBookmark(account_id, bookmark) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account_id).await
                    });
                    block_on(async {
                        match http::remove_bookmark(&account, &bookmark).await {
                            Ok(()) => {
                                let index = self
                                    .bookmarks_view
                                    .bookmarks
                                    .iter()
                                    .position(|x| *x == bookmark)
                                    .unwrap();
                                self.bookmarks_view.bookmarks.remove(index);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "removed-bookmark-from-account",
                                            acc = account.display_name.clone()
                                        )))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                            Err(e) => {
                                log::error!("Error removing bookmark: {}", e);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(format!("{e}")))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                        }
                    });
                    block_on(async {
                        db::SqliteDatabase::delete_bookmark(database, &bookmark).await;
                    });
                }
                block_on(async {
                    self.bookmarks_cursor.refresh_count().await;
                    self.bookmarks_cursor.fetch_next_results().await;
                });
                self.bookmarks_view.bookmarks = self.bookmarks_cursor.result.clone().unwrap();
                self.core.window.show_context = false;
            }
            Message::EditBookmark(account_id, bookmark) => {
                self.placeholder_bookmark = Some(bookmark.clone());
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account_id).await
                    });
                    self.placeholder_account = Some(account);
                };
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::EditBookmarkForm)));
            }
            Message::UpdateBookmark(account, bookmark) => {
                let mut updated_bkmrk: Option<Bookmark> = None;
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        match http::edit_bookmark(&account, &bookmark).await {
                            Ok(value) => {
                                updated_bkmrk = Some(value);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "updated-bookmark-in-account",
                                            acc = account.display_name.clone()
                                        )))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                            Err(e) => {
                                log::error!("Error patching bookmark: {}", e);
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(format!("{e}")))
                                        .map(cosmic::app::Message::App),
                                );
                            }
                        }
                    });
                    if let Some(bkmrk) = updated_bkmrk {
                        let index = self
                            .bookmarks_view
                            .bookmarks
                            .iter()
                            .position(|x| x.id == bookmark.id)
                            .unwrap();
                        block_on(async {
                            db::SqliteDatabase::update_bookmark(database, &bookmark, &bkmrk).await;
                        });
                        self.bookmarks_view.bookmarks[index] = bkmrk;
                    }
                }
                self.core.window.show_context = false;
            }
            Message::SearchBookmarks(query) => {
                if query.is_empty() {
                    self.bookmarks_cursor.search_query = None;
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        self.bookmarks_cursor.refresh_offset(0).await;
                    });
                } else {
                    self.bookmarks_cursor.search_query = Some(query);
                }
                commands.push(self.update(Message::LoadBookmarks));
            }
            Message::OpenExternalUrl(url) => {
                _ = open::that_detached(url);
            }
            Message::ViewBookmarkNotes(bookmark) => {
                self.placeholder_bookmark = Some(bookmark.clone());
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::ViewBookmarkNotes)));
            }
            Message::Key(modifiers, key) => {
                for (key_bind, menu_action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(menu_action.message());
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::OpenRemoveAccountDialog(account_id) => {
                if self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages
                        .push_back(DialogPage::RemoveAccount(account_id));
                }
            }
            Message::OpenRemoveBookmarkDialog(account_id, bookmark) => {
                if self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages
                        .push_back(DialogPage::RemoveBookmark(account_id, bookmark));
                }
            }
            Message::DialogUpdate(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }
            Message::CompleteRemoveDialog(_account, _bookmark) => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::RemoveAccount(account) => {
                            commands.push(self.update(Message::RemoveAccount(account)));
                        }
                        DialogPage::RemoveBookmark(account_id, bookmark) => {
                            commands
                                .push(self.update(Message::RemoveBookmark(account_id, bookmark)));
                        }
                    }
                }
                commands.push(self.update(Message::LoadAccounts));
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::LoadBookmarks => {
                match self.config.sort_option {
                    SortOption::BookmarksDateNewest => {
                        self.bookmarks_cursor.sort_option = SortOption::BookmarksDateNewest;
                    }
                    SortOption::BookmarksDateOldest => {
                        self.bookmarks_cursor.sort_option = SortOption::BookmarksDateOldest;
                    }
                    SortOption::BookmarkAlphabeticalAscending => {
                        self.bookmarks_cursor.sort_option =
                            SortOption::BookmarkAlphabeticalAscending;
                    }
                    SortOption::BookmarkAlphabeticalDescending => {
                        self.bookmarks_cursor.sort_option =
                            SortOption::BookmarkAlphabeticalDescending;
                    }
                }
                block_on(async {
                    self.bookmarks_cursor.fetch_next_results().await;
                    self.bookmarks_cursor.refresh_count().await;
                });
                self.bookmarks_view.bookmarks = self.bookmarks_cursor.result.clone().unwrap();
            }
            Message::IncrementPageIndex(cursor_type) => {
                if cursor_type == "bookmarks" {
                    let current_page = self.bookmarks_cursor.current_page;
                    let total_pages = self.bookmarks_cursor.total_pages;
                    if current_page < total_pages {
                        self.bookmarks_cursor.current_page = current_page + 1;
                    }
                    commands.push(self.update(Message::LoadBookmarks));
                } else if cursor_type == "accounts" {
                    let current_page = self.accounts_cursor.current_page;
                    let total_pages = self.accounts_cursor.total_pages;
                    if current_page < total_pages {
                        self.accounts_cursor.current_page = current_page + 1;
                    }
                    commands.push(self.update(Message::LoadAccounts));
                }
            }
            Message::DecrementPageIndex(cursor_type) => {
                if cursor_type == "bookmarks" {
                    let current_page = self.bookmarks_cursor.current_page;
                    if current_page > 1 {
                        self.bookmarks_cursor.current_page = current_page - 1;
                    }
                    commands.push(self.update(Message::LoadBookmarks));
                } else if cursor_type == "accounts" {
                    let current_page = self.accounts_cursor.current_page;
                    if current_page > 1 {
                        self.accounts_cursor.current_page = current_page - 1;
                    }
                    commands.push(self.update(Message::LoadAccounts));
                }
            }
            Message::StartupCompleted => {
                for account in self.accounts_view.accounts.clone() {
                    commands.push(self.update(Message::StartRefreshAccountProfile(account)));
                }
                commands.push(Task::perform(
                    async {
                        // Initial delay for refresh
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        crate::app::Message::StartRefreshBookmarksForAllAccounts
                    },
                    cosmic::app::Message::App,
                ));
                self.state = ApplicationState::Normal;
            }
            Message::Empty => {
                commands.push(Task::none());
            }
        }
        Task::batch(commands)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        self.update_title()
    }
}

impl Cosmicding {
    #[allow(clippy::unused_self)]
    pub fn about(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let release = env!("CARGO_PKG_VERSION");
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        widget::column::with_children(vec![
            widget::text::title3(fl!("cosmicding")).into(),
            widget::button::link(REPOSITORY)
                .on_press(Message::OpenExternalUrl(REPOSITORY.to_string()))
                .padding(spacing.space_none)
                .into(),
            widget::button::link(fl!(
                "git-description",
                hash = short_hash.as_str(),
                date = date
            ))
            .on_press(Message::OpenExternalUrl(format!(
                "{REPOSITORY}/commits/{hash}"
            )))
            .padding(spacing.space_none)
            .into(),
            widget::text::caption(format!("v{release}")).into(),
        ])
        .align_x(Alignment::Center)
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<Message> {
        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
                .add({
                    let app_theme_selected = match self.config.app_theme {
                        AppTheme::Dark => 1,
                        AppTheme::Light => 2,
                        AppTheme::System => 0,
                    };
                    widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                        &self.app_themes,
                        Some(app_theme_selected),
                        move |index| {
                            Message::AppTheme(match index {
                                1 => AppTheme::Dark,
                                2 => AppTheme::Light,
                                _ => AppTheme::System,
                            })
                        },
                    ))
                })
                .into(),
            widget::settings::section()
                .title(fl!("view"))
                .add({
                    widget::settings::item::builder(fl!(
                        "items-per-page",
                        count = self.config.items_per_page
                    ))
                    .control(widget::slider(
                        5..=50,
                        self.config.items_per_page,
                        Message::SetItemsPerPage,
                    ))
                })
                .into(),
        ])
        .into()
    }

    fn update_config(&mut self) -> Task<Message> {
        let theme = self.config.app_theme.theme();
        cosmic::app::command::set_theme(theme)
    }

    pub fn update_title(&mut self) -> Task<Message> {
        let window_title = match self.nav.text(self.nav.active()) {
            Some(page) => format!("{page} — {}", fl!("cosmicding")),
            _ => fl!("cosmicding"),
        };
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    AddAccountForm,
    EditAccountForm,
    EditBookmarkForm,
    NewBookmarkForm,
    Settings,
    ViewBookmarkNotes,
}

impl ContextPage {
    fn title(self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::AddAccountForm => fl!("add-account"),
            Self::EditAccountForm => fl!("edit-account"),
            Self::NewBookmarkForm => fl!("add-bookmark"),
            Self::EditBookmarkForm => fl!("edit-bookmark"),
            Self::ViewBookmarkNotes => fl!("notes"),
        }
    }
}

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
