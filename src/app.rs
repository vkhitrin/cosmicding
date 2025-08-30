use crate::{
    app::{
        actions::ApplicationAction,
        config::{AppTheme, CosmicConfig, SortOption},
        context::ContextPage,
        dialog::DialogPage,
        menu as app_menu,
        nav::AppNavPage,
    },
    db::{self},
    fl,
    http::{self},
    models::{
        account::{Account, LinkdingAccountApiResponse},
        bookmarks::{
            Bookmark, BookmarkCheckDetailsResponse, BookmarkRemoveResponse, DetailedResponse,
        },
        db_cursor::{AccountsPaginationCursor, BookmarksPaginationCursor, Pagination},
        favicon_cache::Favicon,
        sync_status::SyncStatus,
    },
    pages::{
        accounts::{add_account, edit_account, PageAccountsView},
        bookmarks::{edit_bookmark, new_bookmark, view_notes, PageBookmarksView},
    },
    style::animation::refresh,
};
use cosmic::{
    app::{context_drawer, Core, Task},
    cosmic_config::{self, Update},
    cosmic_theme::{self, ThemeMode},
    iced::{
        event,
        futures::executor::block_on,
        keyboard::{Event as KeyEvent, Modifiers},
        Event, Length, Subscription,
    },
    iced_core::image::Bytes,
    iced_widget::tooltip,
    widget::{
        self,
        about::About,
        icon,
        menu::{key_bind::KeyBind, Action},
        nav_bar,
    },
    Application, ApplicationExt, Element,
};
use cosmic_time::{chain, Timeline};
use key_bind::key_binds;
use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub mod actions;
pub mod config;
pub mod context;
pub mod dialog;
mod key_bind;
pub mod menu;
pub mod nav;

pub const QUALIFIER: &str = "com";
pub const ORG: &str = "vkhitrin";
pub const APP: &str = "cosmicding";
pub const APPID: &str = constcat::concat!(QUALIFIER, ".", ORG, ".", APP);

const REPOSITORY: &str = "https://github.com/vkhitrin/cosmicding";

pub static REFRESH_ICON: std::sync::LazyLock<refresh::Id> =
    std::sync::LazyLock::new(refresh::Id::unique);

pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: CosmicConfig,
}

pub struct Cosmicding {
    about: About,
    app_themes: Vec<String>,
    config_handler: Option<cosmic_config::Config>,
    context_page: ContextPage,
    core: Core,
    dialog_pages: VecDeque<DialogPage>,
    key_binds: HashMap<KeyBind, menu::MenuAction>,
    modifiers: Modifiers,
    nav: nav_bar::Model,
    placeholder_account: Option<Account>,
    placeholder_accounts_list: Vec<Account>,
    placeholder_bookmark: Option<Bookmark>,
    placeholder_bookmark_description: widget::text_editor::Content,
    placeholder_bookmark_notes: widget::text_editor::Content,
    placeholder_selected_account_index: usize,
    pub accounts_cursor: AccountsPaginationCursor,
    pub accounts_view: PageAccountsView,
    pub bookmarks_cursor: BookmarksPaginationCursor,
    pub bookmarks_view: PageBookmarksView,
    pub config: CosmicConfig,
    pub state: ApplicationState,
    search_id: widget::Id,
    timeline: Timeline,
    sync_status: SyncStatus,
    toasts: widget::toaster::Toasts<ApplicationAction>,
}

#[derive(Debug, Clone, Copy)]
pub enum ApplicationState {
    Loading,
    NoEnabledAccounts,
    Ready,
    Refreshing,
}

impl Application for Cosmicding {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = ApplicationAction;

    const APP_ID: &'static str = APPID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let timeline = Timeline::new();
        let db_pool = Some(block_on(async {
            db::SqliteDatabase::create().await.unwrap()
        }));
        let accounts_cursor = AccountsPaginationCursor::new(db_pool.clone().unwrap());
        let bookmarks_cursor = BookmarksPaginationCursor::new(db_pool.clone().unwrap());
        let mut nav = nav_bar::Model::default();
        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        let release = env!("CARGO_PKG_VERSION");
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let about = About::default()
            .name(fl!("cosmicding"))
            .icon(icon::from_name(self::APPID))
            .version(release)
            .comments(fl!(
                "git-description",
                hash = short_hash.as_str(),
                date = date
            ))
            .license("GPL-3.0")
            .author("Vadim Khitrin")
            .links([
                ("Repository", REPOSITORY),
                ("Support", &format!("{REPOSITORY}/issues")),
                ("Linkding Official Site", "https://linkding.link"),
            ])
            .translators([("Luna Jernberg", "lunajernberg@gnome.org")])
            .developers([("Vadim Khitrin", "me@vkhitrin.com")]);

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
            about,
            accounts_cursor,
            accounts_view: PageAccountsView::default(),
            app_themes,
            bookmarks_cursor,
            bookmarks_view: PageBookmarksView::default(),
            config: flags.config,
            config_handler: flags.config_handler,
            context_page: ContextPage::Settings,
            core,
            dialog_pages: VecDeque::new(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            nav,
            placeholder_account: None,
            placeholder_accounts_list: Vec::new(),
            placeholder_bookmark: None,
            placeholder_bookmark_description: widget::text_editor::Content::new(),
            placeholder_bookmark_notes: widget::text_editor::Content::new(),
            placeholder_selected_account_index: 0,
            state: ApplicationState::NoEnabledAccounts,
            search_id: widget::Id::unique(),
            timeline,
            sync_status: SyncStatus::default(),
            toasts: widget::toaster::Toasts::new(ApplicationAction::CloseToast),
        };

        app.bookmarks_cursor.items_per_page = app.config.items_per_page;
        app.accounts_cursor.items_per_page = app.config.items_per_page;
        // NOTE: (vkhitrin) probably wiser to initiate this field in the constructor above
        app.bookmarks_view.search_id = Some(app.search_id.clone());

        let commands = vec![
            app.update_title(),
            app.update(ApplicationAction::SetItemsPerPage(
                app.config.items_per_page,
            )),
            app.update(ApplicationAction::StartupCompleted),
        ];

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            app.bookmarks_cursor.refresh_count().await;
        });
        app.timeline.set_chain(chain![REFRESH_ICON]).start();

        (app, Task::batch(commands))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![app_menu::menu_bar(
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

    fn on_escape(&mut self) -> Task<ApplicationAction> {
        if self.dialog_pages.pop_front().is_some() {
            return Task::none();
        }

        self.core.window.show_context = false;
        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<ApplicationAction>> {
        if !self.core.window.show_context {
            return None;
        }
        match self.context_page {
            ContextPage::About => Some(
                context_drawer::about(
                    &self.about,
                    ApplicationAction::OpenExternalUrl,
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::Settings => Some(
                context_drawer::context_drawer(self.settings(), ApplicationAction::ContextClose)
                    .title(self.context_page.title()),
            ),
            ContextPage::AddAccountForm => Some(
                context_drawer::context_drawer(
                    add_account(self.placeholder_account.clone().unwrap()),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::EditAccountForm => Some(
                context_drawer::context_drawer(
                    edit_account(self.placeholder_account.clone().unwrap()),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::NewBookmarkForm => Some(
                context_drawer::context_drawer(
                    new_bookmark(
                        self.placeholder_bookmark.clone().unwrap(),
                        &self.placeholder_bookmark_notes,
                        &self.placeholder_bookmark_description,
                        &self.placeholder_accounts_list,
                        self.placeholder_selected_account_index,
                    ),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::EditBookmarkForm => Some(
                context_drawer::context_drawer(
                    edit_bookmark(
                        self.placeholder_bookmark.clone().unwrap(),
                        &self.placeholder_bookmark_notes,
                        &self.placeholder_bookmark_description,
                        self.placeholder_account.as_ref().unwrap(),
                    ),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::ViewBookmarkNotes => Some(
                context_drawer::context_drawer(
                    view_notes(&self.placeholder_bookmark_notes),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
        }
    }

    fn dialog(&self) -> Option<Element<ApplicationAction>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::RemoveAccount(account) => widget::dialog()
                .title(fl!("remove") + " " + { &account.display_name })
                .icon(icon::from_name("dialog-warning-symbolic").size(58))
                .body(fl!("remove-account-confirm"))
                .primary_action(widget::button::destructive(fl!("yes")).on_press_maybe(Some(
                    ApplicationAction::CompleteRemoveDialog(account.id, None),
                )))
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(ApplicationAction::DialogCancel),
                ),
            DialogPage::RemoveBookmark(account, bookmark) => widget::dialog()
                .icon(icon::from_name("dialog-warning-symbolic").size(58))
                .title(fl!("remove") + " " + { &bookmark.title })
                .body(fl!("remove-bookmark-confirm"))
                .primary_action(widget::button::destructive(fl!("yes")).on_press_maybe(Some(
                    ApplicationAction::CompleteRemoveDialog(Some(*account), Some(bookmark.clone())),
                )))
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(ApplicationAction::DialogCancel),
                ),
            DialogPage::PurgeFaviconsCache() => widget::dialog()
                .icon(icon::from_name("dialog-warning-symbolic").size(58))
                .title(fl!("purge-favicons-cache"))
                .body(fl!("purge-favicons-cache-confirm"))
                .primary_action(
                    widget::button::destructive(fl!("yes"))
                        .on_press_maybe(Some(ApplicationAction::CompleteRemoveDialog(None, None))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(ApplicationAction::DialogCancel),
                ),
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
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status, _| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(ApplicationAction::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(ApplicationAction::Modifiers(modifiers))
                }
                _ => None,
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
                ApplicationAction::SystemThemeModeChange
            }),
            // NOTE: (vkhitrin) native implementation is too resource heavy.
            // self.timeline
            //     .as_subscription()
            //     .map(|(_id, instant)| ApplicationAction::Tick(instant)),
            cosmic::iced::time::every(Duration::from_millis(250)).map(ApplicationAction::Tick),
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
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }
        match message {
            #![allow(non_snake_case)]
            ApplicationAction::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            ApplicationAction::SortOption(sort_option) => {
                config_set!(sort_option, sort_option);
                if !self.bookmarks_view.bookmarks.is_empty() {
                    return self.update(ApplicationAction::LoadBookmarks);
                }
            }
            ApplicationAction::SetItemsPerPage(items_per_page) => {
                config_set!(items_per_page, items_per_page);
                self.bookmarks_cursor.items_per_page = items_per_page;
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    self.accounts_cursor.refresh_count().await;
                    self.accounts_cursor.refresh_offset(0).await;
                    self.bookmarks_cursor.refresh_count().await;
                    self.bookmarks_cursor.refresh_offset(0).await;
                });
                commands.push(self.update(ApplicationAction::LoadAccounts));
                commands.push(self.update(ApplicationAction::LoadBookmarks));
            }
            ApplicationAction::SystemThemeModeChange => {
                return self.update_config();
            }
            ApplicationAction::EnableFavicons(enable_favicon) => {
                config_set!(enable_favicons, enable_favicon);
                self.config.enable_favicons = enable_favicon;
            }
            ApplicationAction::OpenAccountsPage => {
                let account_page_entity = &self.nav.entity_at(0);
                self.nav.activate(account_page_entity.unwrap());
            }
            ApplicationAction::Tick(now) => {
                self.timeline.now(now);
            }
            ApplicationAction::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            ApplicationAction::ContextClose => self.core.window.show_context = false,
            ApplicationAction::AccountsView(message) => {
                commands.push(self.accounts_view.update(message));
            }
            ApplicationAction::LoadAccounts => {
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
            ApplicationAction::AddAccountForm => {
                self.placeholder_account =
                    Some(Account::new(String::new(), String::new(), String::new()));
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::AddAccountForm,
                )));
            }
            ApplicationAction::EditAccountForm(account) => {
                self.placeholder_account = Some(account.clone());
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::EditAccountForm,
                )));
            }
            ApplicationAction::RemoveAccount(account) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        db::SqliteDatabase::delete_all_favicons_cache_of_account(
                            database,
                            account.id.unwrap(),
                        )
                        .await;
                    });
                    block_on(async {
                        db::SqliteDatabase::delete_all_bookmarks_of_account(
                            database,
                            account.id.unwrap(),
                        )
                        .await;
                    });
                    block_on(async {
                        db::SqliteDatabase::delete_account(database, account.id.unwrap()).await;
                    });
                    self.bookmarks_view
                        .bookmarks
                        .retain(|bkmrk| bkmrk.user_account_id != Some(account.id.unwrap()));
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "removed-account",
                                acc = account.display_name
                            )))
                            .map(cosmic::Action::App),
                    );
                    block_on(async {
                        self.accounts_cursor.refresh_count().await;
                        self.accounts_cursor.fetch_next_results().await;
                    });
                    self.accounts_view.accounts = self.accounts_cursor.result.clone().unwrap();
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                }
            }
            ApplicationAction::StartAddAccount(account) => {
                if let Some(ref mut database) = &mut self.accounts_cursor.database {
                    let account_exists = block_on(async {
                        db::SqliteDatabase::check_if_account_exists(
                            database,
                            &account.instance,
                            &account.api_token,
                        )
                        .await
                    });
                    if account_exists {
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!("account-exists")))
                                .map(cosmic::Action::App),
                        );
                    } else {
                        let cloned_acc = account.clone();
                        let message = move |api_response: Option<LinkdingAccountApiResponse>| {
                            cosmic::Action::App(ApplicationAction::DoneAddAccount(
                                cloned_acc.clone(),
                                api_response,
                            ))
                        };
                        commands.push(Task::perform(http::fetch_account_details(account), message));
                    }
                }
            }
            ApplicationAction::DoneAddAccount(account, api_response) => {
                if let Some(response) = api_response {
                    if response.error.is_none() {
                        if let Some(ref mut database) = &mut self.accounts_cursor.database {
                            block_on(async {
                                db::SqliteDatabase::create_account(database, &account).await;
                            });
                            commands.push(self.update(ApplicationAction::LoadAccounts));
                            commands.push(self.update(
                                ApplicationAction::StartRefreshBookmarksForAccount(
                                    self.accounts_view.accounts.last().unwrap().clone(),
                                ),
                            ));
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!(
                                        "added-account",
                                        acc = account.display_name
                                    )))
                                    .map(cosmic::Action::App),
                            );
                        }
                    } else {
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "failed-to-add-account",
                                    acc = account.display_name,
                                    err = response.error
                                )))
                                .map(cosmic::Action::App),
                        );
                    }
                }
                self.core.window.show_context = false;
            }
            ApplicationAction::StartEditAccount(account) => {
                let cloned_acc = account.clone();
                if account.enabled {
                    let message = move |api_response: Option<LinkdingAccountApiResponse>| {
                        cosmic::Action::App(ApplicationAction::DoneEditAccount(
                            cloned_acc.clone(),
                            api_response,
                        ))
                    };
                    commands.push(Task::perform(http::fetch_account_details(account), message));
                } else {
                    commands.push(self.update(ApplicationAction::DoneEditAccount(account, None)));
                }
                self.core.window.show_context = false;
            }
            ApplicationAction::DoneEditAccount(mut account, api_response) => {
                let account_clone = account.clone();
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let current_account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account.id.unwrap())
                            .await
                    });
                    if let Some(response) = api_response {
                        if response.error.is_none() {
                            account.enable_public_sharing = response.enable_public_sharing;
                            account.enable_sharing = response.enable_sharing;
                            if current_account != account {
                                block_on(async {
                                    db::SqliteDatabase::update_account(database, &account).await;
                                });
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "updated-account",
                                            acc = account_clone.display_name
                                        )))
                                        .map(cosmic::Action::App),
                                );
                                commands.push(self.update(ApplicationAction::LoadAccounts));
                                if account.requires_remote_sync(&current_account) {
                                    commands.push(self.update(
                                        ApplicationAction::StartRefreshBookmarksForAccount(
                                            account.clone(),
                                        ),
                                    ));
                                }
                            }
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!(
                                        "failed-to-edit-account",
                                        acc = account.display_name,
                                        err = response.error
                                    )))
                                    .map(cosmic::Action::App),
                            );
                        }
                    } else if current_account != account {
                        block_on(async {
                            db::SqliteDatabase::update_account(database, &account).await;
                        });
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "disabled-account",
                                    acc = account_clone.display_name
                                )))
                                .map(cosmic::Action::App),
                        );
                        commands.push(self.update(ApplicationAction::LoadAccounts));
                        commands.push(self.update(
                            ApplicationAction::StartRefreshBookmarksForAccount(account.clone()),
                        ));
                    }
                }
            }
            ApplicationAction::BookmarksView(message) => {
                commands.push(self.bookmarks_view.update(message));
            }
            ApplicationAction::StartRefreshBookmarksForAllAccounts => {
                if !self.accounts_view.accounts.is_empty() {
                    if let ApplicationState::Refreshing = self.state {
                    } else {
                        // NOTE: (vkhitrin) if all accounts disabled, do not attempt to refresh
                        if self.accounts_view.accounts.iter().all(|item| !item.enabled) {
                            self.state = ApplicationState::NoEnabledAccounts;
                        } else {
                            self.state = ApplicationState::Refreshing;
                            let message = |x: Vec<DetailedResponse>| {
                                cosmic::Action::App(
                                    ApplicationAction::DoneRefreshBookmarksForAllAccounts(x),
                                )
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
                }
            }
            ApplicationAction::DoneRefreshBookmarksForAllAccounts(remote_responses) => {
                let mut failed_accounts: Vec<String> = Vec::new();
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    for response in remote_responses.iter().cloned() {
                        if !response.successful {
                            failed_accounts.push(response.account.display_name.clone());
                        }
                        block_on(async {
                            db::SqliteDatabase::aggregate_bookmarks_for_account(
                                database,
                                &response.account,
                                response.bookmarks.unwrap_or_else(Vec::new),
                                response.timestamp,
                                response.successful,
                            )
                            .await;
                        });
                    }
                    commands.push(self.update(ApplicationAction::LoadAccounts));
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                    self.state = ApplicationState::Ready;
                    if failed_accounts.is_empty() {
                        self.sync_status = SyncStatus::Successful;
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!("refreshed-bookmarks")))
                                .map(cosmic::Action::App),
                        );
                    } else if remote_responses.len() == failed_accounts.len() {
                        self.sync_status = SyncStatus::Failed;
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "failed-refreshing-all-accounts"
                                )))
                                .map(cosmic::Action::App),
                        );
                    } else {
                        self.sync_status = SyncStatus::Warning;
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "failed-refreshing-accounts",
                                    accounts = failed_accounts.join(", ")
                                )))
                                .map(cosmic::Action::App),
                        );
                    }
                }
            }
            ApplicationAction::StartRefreshBookmarksForAccount(account) => {
                if let ApplicationState::Refreshing = self.state {
                } else if account.enabled {
                    self.state = ApplicationState::Refreshing;
                    let mut acc_vec = self.accounts_view.accounts.clone();
                    acc_vec.retain(|acc| acc.id == account.id);
                    let cloned_acc = acc_vec[0].clone();
                    let message = move |bookmarks: Vec<DetailedResponse>| {
                        cosmic::Action::App(ApplicationAction::DoneRefreshBookmarksForAccount(
                            cloned_acc.clone(),
                            bookmarks,
                        ))
                    };
                    commands.push(self.update(ApplicationAction::StartRefreshAccountProfile(
                        account.clone(),
                    )));
                    if !acc_vec.is_empty() {
                        commands.push(Task::perform(
                            http::fetch_bookmarks_from_all_accounts(acc_vec.clone()),
                            message,
                        ));
                    }
                } else {
                    if self.accounts_view.accounts.iter().all(|item| !item.enabled) {
                        self.state = ApplicationState::NoEnabledAccounts;
                    }
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                }
            }
            ApplicationAction::DoneRefreshBookmarksForAccount(account, remote_responses) => {
                let mut failure_refreshing = false;
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    for response in remote_responses {
                        if !response.successful {
                            failure_refreshing = true;
                        }
                        block_on(async {
                            db::SqliteDatabase::aggregate_bookmarks_for_account(
                                database,
                                &account,
                                response.bookmarks.unwrap_or_else(Vec::new),
                                response.timestamp,
                                response.successful,
                            )
                            .await;
                        });
                    }
                    commands.push(self.update(ApplicationAction::LoadAccounts));
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                    self.state = ApplicationState::Ready;
                    if failure_refreshing {
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "failed-refreshing-bookmarks-for-account",
                                    account = account.display_name
                                )))
                                .map(cosmic::Action::App),
                        );
                    } else {
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "refreshed-bookmarks-for-account",
                                    acc = account.display_name
                                )))
                                .map(cosmic::Action::App),
                        );
                    }
                }
            }
            ApplicationAction::StartRefreshAccountProfile(account) => {
                if let ApplicationState::Refreshing = self.state {
                } else if account.enabled {
                    self.state = ApplicationState::Refreshing;
                    let cloned_acc = account.clone();
                    let message = move |api_response: Option<LinkdingAccountApiResponse>| {
                        cosmic::Action::App(ApplicationAction::DoneRefreshAccountProfile(
                            cloned_acc.clone(),
                            api_response,
                        ))
                    };
                    commands.push(Task::perform(http::fetch_account_details(account), message));
                }
            }
            ApplicationAction::DoneRefreshAccountProfile(mut account, api_response) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if let Some(response) = api_response {
                        if response.successful.expect("") {
                            account.enable_sharing = response.enable_sharing;
                            account.enable_public_sharing = response.enable_public_sharing;
                            block_on(async {
                                db::SqliteDatabase::update_account(database, &account).await;
                            });
                            commands.push(self.update(ApplicationAction::LoadAccounts));
                        }
                    }
                    self.state = ApplicationState::Loading;
                    self.sync_status = SyncStatus::InProgress;
                }
            }
            ApplicationAction::AddBookmarkForm => {
                if matches!(self.state, ApplicationState::Ready) {
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
                            Some(true),
                        ));
                        if !self.placeholder_bookmark_notes.text().is_empty() {
                            self.placeholder_bookmark_notes = widget::text_editor::Content::new();
                        }
                        if !self.placeholder_bookmark_description.text().is_empty() {
                            self.placeholder_bookmark_description =
                                widget::text_editor::Content::new();
                        }
                        commands.push(self.update(ApplicationAction::ToggleContextPage(
                            ContextPage::NewBookmarkForm,
                        )));
                    }
                }
            }
            ApplicationAction::SetAccountDisplayName(name) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.display_name = name;
                }
            }
            ApplicationAction::SetAccountInstance(url) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.instance = url;
                }
            }
            ApplicationAction::SetAccountAPIKey(key) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.api_token = key;
                }
            }
            ApplicationAction::SetAccountStatus(enabled) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.enabled = enabled;
                }
            }
            ApplicationAction::SetAccountTrustInvalidCertificates(trust) => {
                if let Some(ref mut account_placeholder) = &mut self.placeholder_account {
                    account_placeholder.trust_invalid_certs = trust;
                }
            }
            ApplicationAction::AddBookmarkFormAccountIndex(idx) => {
                self.placeholder_selected_account_index = idx;
            }
            ApplicationAction::SetBookmarkURL(url) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.url = url;
                }
            }
            ApplicationAction::SetBookmarkTitle(title) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.title = title;
                }
            }
            ApplicationAction::InputBookmarkNotes(action) => {
                self.placeholder_bookmark_notes.perform(action);
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.notes = self.placeholder_bookmark_notes.text();
                }
            }
            ApplicationAction::InputBookmarkDescription(action) => {
                self.placeholder_bookmark_description.perform(action);
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.description = self.placeholder_bookmark_description.text();
                }
            }
            ApplicationAction::SetBookmarkTags(tags_string) => {
                let tags: Vec<String> = if tags_string.is_empty() {
                    Vec::new()
                } else {
                    tags_string
                        .split(' ')
                        .map(|s| s.trim().to_string())
                        .collect()
                };
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.tag_names = tags;
                }
            }
            ApplicationAction::SetBookmarkArchived(archived) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.is_archived = archived;
                }
            }
            ApplicationAction::SetBookmarkUnread(unread) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.unread = unread;
                }
            }
            ApplicationAction::SetBookmarkShared(shared) => {
                if let Some(ref mut bookmark_placeholder) = &mut self.placeholder_bookmark {
                    bookmark_placeholder.shared = shared;
                }
            }
            // NOTE: (vkhitrin) during creation, linkding doesn't populate 'favicon_url'.
            //       In order to display the new favicon, users are required to wait a
            //       bit, and then perform a manual refresh.
            ApplicationAction::StartAddBookmark(account, bookmark) => {
                let cloned_acc = account.clone();
                let message = move |api_response: Option<BookmarkCheckDetailsResponse>| {
                    cosmic::Action::App(ApplicationAction::DoneAddBookmark(
                        cloned_acc.clone(),
                        api_response,
                    ))
                };
                commands.push(Task::perform(
                    http::populate_bookmark(account, bookmark, true),
                    message,
                ));
                self.core.window.show_context = false;
            }
            ApplicationAction::DoneAddBookmark(account, api_response) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if let Some(response) = api_response {
                        if response.error.is_none() {
                            if let Some(mut bkmrk) = response.bookmark {
                                bkmrk.is_owner = Some(true);
                                if response.is_new {
                                    block_on(async {
                                        db::SqliteDatabase::add_bookmark(database, &bkmrk).await;
                                    });
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "added-bookmark-to-account",
                                                bkmrk = bkmrk.url.clone(),
                                                acc = account.display_name
                                            )))
                                            .map(cosmic::Action::App),
                                    );
                                } else {
                                    block_on(async {
                                        db::SqliteDatabase::update_bookmark(
                                            database, &bkmrk, &bkmrk,
                                        )
                                        .await;
                                    });
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "updated-bookmark-in-account",
                                                bkmrk = bkmrk.url,
                                                acc = account.display_name.clone()
                                            )))
                                            .map(cosmic::Action::App),
                                    );
                                }
                                commands.push(self.update(ApplicationAction::LoadBookmarks));
                            }
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(response.error.unwrap()))
                                    .map(cosmic::Action::App),
                            );
                        }
                    }
                }
            }
            ApplicationAction::StartRemoveBookmark(account_id, bookmark) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account_id).await
                    });
                    let cloned_account = account.clone();
                    let cloned_bookmark = bookmark.clone();
                    let message = move |api_response: Option<BookmarkRemoveResponse>| {
                        cosmic::Action::App(ApplicationAction::DoneRemoveBookmark(
                            cloned_account.clone(),
                            cloned_bookmark.clone(),
                            api_response,
                        ))
                    };
                    commands.push(Task::perform(
                        http::remove_bookmark(account, bookmark),
                        message,
                    ));
                }
                self.core.window.show_context = false;
            }
            ApplicationAction::DoneRemoveBookmark(account, bookmark, api_response) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if let Some(response) = api_response {
                        if response.error.is_none() {
                            block_on(async {
                                db::SqliteDatabase::delete_bookmark(database, &bookmark).await;
                            });
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
                                    .map(cosmic::Action::App),
                            );

                            commands.push(self.update(ApplicationAction::LoadBookmarks));
                            self.bookmarks_view.bookmarks =
                                self.bookmarks_cursor.result.clone().unwrap();
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(response.error.unwrap()))
                                    .map(cosmic::Action::App),
                            );
                        }
                    }
                }
            }
            ApplicationAction::EditBookmarkForm(account_id, bookmark) => {
                self.placeholder_bookmark = Some(bookmark.clone());
                self.placeholder_bookmark_notes = widget::text_editor::Content::with_text(
                    &self.placeholder_bookmark.as_ref().unwrap().notes,
                );
                self.placeholder_bookmark_description = widget::text_editor::Content::with_text(
                    &self.placeholder_bookmark.as_ref().unwrap().description,
                );
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account_id).await
                    });
                    self.placeholder_account = Some(account);
                }
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::EditBookmarkForm,
                )));
            }
            ApplicationAction::StartEditBookmark(account, bookmark) => {
                let cloned_acc = account.clone();
                let message = move |api_response: Option<BookmarkCheckDetailsResponse>| {
                    cosmic::Action::App(ApplicationAction::DoneAddBookmark(
                        cloned_acc.clone(),
                        api_response,
                    ))
                };
                commands.push(Task::perform(
                    http::populate_bookmark(account, bookmark, false),
                    message,
                ));
                self.core.window.show_context = false;
            }
            ApplicationAction::DoneEditBookmark(account, api_response) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if let Some(response) = api_response {
                        if response.error.is_none() {
                            if let Some(mut bkmrk) = response.bookmark {
                                bkmrk.is_owner = Some(true);
                                block_on(async {
                                    db::SqliteDatabase::update_bookmark(database, &bkmrk, &bkmrk)
                                        .await;
                                });
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "updated-bookmark-in-account",
                                            bkmrk = bkmrk.url,
                                            acc = account.display_name.clone()
                                        )))
                                        .map(cosmic::Action::App),
                                );
                                commands.push(self.update(ApplicationAction::LoadBookmarks));
                            }
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(response.error.unwrap()))
                                    .map(cosmic::Action::App),
                            );
                        }
                    }
                }
            }
            ApplicationAction::SearchBookmarks(search_query) => {
                if search_query.is_empty() {
                    self.bookmarks_cursor.search_query = None;
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        self.bookmarks_cursor.refresh_offset(0).await;
                    });
                } else {
                    self.bookmarks_cursor.search_query = Some(search_query);
                }
                commands.push(self.update(ApplicationAction::LoadBookmarks));
            }
            ApplicationAction::OpenExternalUrl(ref url) => {
                if let Err(err) = open::that_detached(url) {
                    log::error!("Failed to open URL: {err}");
                }
            }
            ApplicationAction::ViewBookmarkNotes(bookmark) => {
                self.placeholder_bookmark_notes =
                    widget::text_editor::Content::with_text(&bookmark.notes);
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::ViewBookmarkNotes,
                )));
            }
            ApplicationAction::Key(modifiers, key) => {
                for (key_bind, menu_action) in &self.key_binds {
                    let menu_message = menu_action.message();
                    if key_bind.matches(modifiers, &key) {
                        return self.update(menu_message);
                    }
                }
            }
            ApplicationAction::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            ApplicationAction::UpdateConfig(config) => {
                self.config = config;
            }
            ApplicationAction::OpenRemoveAccountDialog(account) => {
                if self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages
                        .push_back(DialogPage::RemoveAccount(account));
                }
            }
            ApplicationAction::OpenRemoveBookmarkDialog(account_id, bookmark) => {
                if self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages
                        .push_back(DialogPage::RemoveBookmark(account_id, bookmark));
                }
            }
            ApplicationAction::OpenPurgeFaviconsCache => {
                if self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages
                        .push_back(DialogPage::PurgeFaviconsCache());
                }
            }
            ApplicationAction::DialogUpdate(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }
            ApplicationAction::CompleteRemoveDialog(_account, _bookmark) => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::RemoveAccount(account) => {
                            commands.push(self.update(ApplicationAction::RemoveAccount(account)));
                        }
                        DialogPage::RemoveBookmark(account_id, bookmark) => {
                            commands.push(self.update(ApplicationAction::StartRemoveBookmark(
                                account_id, bookmark,
                            )));
                        }
                        DialogPage::PurgeFaviconsCache() => {
                            commands.push(self.update(ApplicationAction::PurgeFaviconsCache));
                            commands.push(self.update(ApplicationAction::LoadBookmarks));
                        }
                    }
                }
                commands.push(self.update(ApplicationAction::LoadAccounts));
            }
            ApplicationAction::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            ApplicationAction::CloseToast(id) => {
                self.toasts.remove(id);
            }
            ApplicationAction::LoadBookmarks => {
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
                // TODO: (vkhitrin) Check favicon cached timestamp, and refresh periodically.
                if self.config.enable_favicons {
                    for bookmark in self.bookmarks_cursor.result.clone().unwrap() {
                        commands.push(
                            self.update(ApplicationAction::StartFetchFaviconForBookmark(bookmark)),
                        );
                    }
                }
                self.bookmarks_view.bookmarks = self.bookmarks_cursor.result.clone().unwrap();
            }
            ApplicationAction::IncrementPageIndex(cursor_type) => {
                if cursor_type == "bookmarks" {
                    let current_page = self.bookmarks_cursor.current_page;
                    let total_pages = self.bookmarks_cursor.total_pages;
                    if current_page < total_pages {
                        self.bookmarks_cursor.current_page = current_page + 1;
                    }
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                } else if cursor_type == "accounts" {
                    let current_page = self.accounts_cursor.current_page;
                    let total_pages = self.accounts_cursor.total_pages;
                    if current_page < total_pages {
                        self.accounts_cursor.current_page = current_page + 1;
                    }
                    commands.push(self.update(ApplicationAction::LoadAccounts));
                }
            }
            ApplicationAction::DecrementPageIndex(cursor_type) => {
                if cursor_type == "bookmarks" {
                    let current_page = self.bookmarks_cursor.current_page;
                    if current_page > 1 {
                        self.bookmarks_cursor.current_page = current_page - 1;
                    }
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
                } else if cursor_type == "accounts" {
                    let current_page = self.accounts_cursor.current_page;
                    if current_page > 1 {
                        self.accounts_cursor.current_page = current_page - 1;
                    }
                    commands.push(self.update(ApplicationAction::LoadAccounts));
                }
            }
            ApplicationAction::StartFetchFaviconForBookmark(bookmark) => {
                if let Some(favicon_url) = bookmark.favicon_url.clone() {
                    if !favicon_url.is_empty() {
                        if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                            let favicon_url_clone = favicon_url.clone();
                            block_on(async {
                                let existing_favicon_opt =
                                    db::SqliteDatabase::check_if_favicon_cache_exists(
                                        database,
                                        &favicon_url_clone,
                                    )
                                    .await;
                                let should_fetch = match &existing_favicon_opt {
                                    Ok(existing_favicon) => {
                                        let now = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                            as i64;

                                        let age = now - existing_favicon.last_sync_timestamp;
                                        if existing_favicon.favicon_data.is_empty() {
                                            log::info!(
                                                "Attempting to refetch previously failed favicon."
                                            );
                                            age > 3600
                                        } else {
                                            log::info!("Favicon is stale, attempting to refetch.");
                                            age > 86400
                                        }
                                    }
                                    Err(_) => true,
                                };
                                if should_fetch {
                                    let message = move |b: Bytes| {
                                        cosmic::Action::App(
                                            ApplicationAction::DoneFetchFaviconForBookmark(
                                                favicon_url.clone(),
                                                b,
                                            ),
                                        )
                                    };
                                    commands.push(Task::perform(
                                        http::fetch_bookmark_favicon(favicon_url_clone.clone()),
                                        message,
                                    ));
                                }
                            });
                        }
                    }
                }
            }
            ApplicationAction::DoneFetchFaviconForBookmark(favicon_url, bytes) => {
                let epoch_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("SystemTime before UNIX EPOCH!")
                    .as_secs();

                let favicon = if bytes.is_empty() {
                    log::warn!("Failed to fetch favicon from {favicon_url:?}");
                    Favicon::new(favicon_url.clone(), vec![], epoch_timestamp as i64)
                } else {
                    Favicon::new(favicon_url.clone(), bytes.to_vec(), epoch_timestamp as i64)
                };

                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        db::SqliteDatabase::add_favicon_cache(database, favicon).await;
                    });
                }
                commands.push(self.update(ApplicationAction::LoadBookmarks));
            }
            ApplicationAction::PurgeFaviconsCache => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    block_on(async {
                        db::SqliteDatabase::purge_favicons_cache(database).await;
                    });
                }
            }
            ApplicationAction::StartupCompleted => {
                for account in self.accounts_view.accounts.clone() {
                    commands
                        .push(self.update(ApplicationAction::StartRefreshAccountProfile(account)));
                }
                commands.push(Task::perform(
                    async {
                        // Initial delay for refresh
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        crate::app::ApplicationAction::StartRefreshBookmarksForAllAccounts
                    },
                    cosmic::Action::App,
                ));
            }
            ApplicationAction::SearchActivate => {
                if let ApplicationState::NoEnabledAccounts = self.state {
                } else {
                    return widget::text_input::focus(self.search_id.clone());
                }
            }
            ApplicationAction::Empty => {
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
    fn settings(&self) -> Element<ApplicationAction> {
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
                            ApplicationAction::AppTheme(match index {
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
                        ApplicationAction::SetItemsPerPage,
                    ))
                })
                // NOTE: (vkhitrin) it is possible to use the native implementation for settings
                // toggler, 'widget::settings::item::builder().toggler()', but it doesn't
                // implement tooltip.
                .add(
                    widget::row::with_capacity(2)
                        .align_y(cosmic::iced::Alignment::Center)
                        .spacing(5)
                        .push(widget::text::body(fl!("enable-favicons")))
                        .push(widget::tooltip(
                            widget::icon::from_name("dialog-information-symbolic").size(18),
                            widget::container(widget::text::body(fl!("enable-favicons-info"))),
                            tooltip::Position::FollowCursor,
                        ))
                        .push(widget::horizontal_space())
                        .push(
                            widget::toggler(self.config.enable_favicons)
                                .on_toggle(ApplicationAction::EnableFavicons),
                        ),
                )
                .into(),
            widget::settings::section()
                .title(fl!("actions"))
                .add(
                    widget::row::with_capacity(2)
                        .align_y(cosmic::iced::Alignment::Center)
                        .spacing(5)
                        .push(widget::text::body(fl!("purge-favicons-cache")))
                        .push(widget::horizontal_space())
                        .push(
                            widget::button::icon(widget::icon::from_name("user-trash-symbolic"))
                                .on_press(ApplicationAction::OpenPurgeFaviconsCache)
                                .class(cosmic::style::Button::Destructive),
                        ),
                )
                .into(),
        ])
        .into()
    }

    fn update_config(&mut self) -> Task<ApplicationAction> {
        let theme = self.config.app_theme.theme();
        cosmic::command::set_theme(theme)
    }

    pub fn update_title(&mut self) -> Task<ApplicationAction> {
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
