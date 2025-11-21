use crate::{
    app::{
        actions::{ApplicationAction, ImportAction},
        config::{AppTheme, CosmicConfig, SortOption},
        context::ContextPage,
        dialog::DialogPage,
        menu as app_menu,
        nav::AppNavPage,
    },
    db::{self},
    fl,
    models::{
        account::{Account, LinkdingAccountApiResponse},
        bookmarks::{
            Bookmark, BookmarkCheckDetailsResponse, BookmarkRemoveResponse, DetailedResponse,
        },
        db_cursor::{AccountsPaginationCursor, BookmarksPaginationCursor, Pagination},
        favicon_cache::Favicon,
        operation::OperationProgress,
        provider::Provider,
        sync_status::SyncStatus,
    },
    pages::{
        accounts::{add_account, edit_account, PageAccountsView},
        bookmarks::{edit_bookmark, new_bookmark, view_notes, PageBookmarksView},
    },
    provider::{self},
    style::animation::refresh,
    utils::bookmark_parser,
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
    theme,
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
    path::PathBuf,
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

async fn open_save_file_dialog(default_name: &str) -> Option<PathBuf> {
    use rfd::AsyncFileDialog;

    AsyncFileDialog::new()
        .set_file_name(default_name)
        .add_filter("HTML Files", &["html"])
        .save_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn open_file_dialog() -> Option<PathBuf> {
    use rfd::AsyncFileDialog;

    AsyncFileDialog::new()
        .add_filter("HTML Files", &["html"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

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
    context_account: Option<Account>,
    context_accounts_list: Vec<Account>,
    context_bookmark: Option<Bookmark>,
    context_bookmark_description: widget::text_editor::Content,
    context_bookmark_notes: widget::text_editor::Content,
    context_selected_account_index: usize,
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
    operation_progress: Option<OperationProgress>,
}

#[derive(Debug, Clone, Copy)]
pub enum ApplicationState {
    Loading,
    NoEnabledRemoteAccounts,
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
            context_account: None,
            context_accounts_list: Vec::new(),
            context_bookmark: None,
            context_bookmark_description: widget::text_editor::Content::new(),
            context_bookmark_notes: widget::text_editor::Content::new(),
            context_selected_account_index: 0,
            state: ApplicationState::NoEnabledRemoteAccounts,
            search_id: widget::Id::unique(),
            timeline,
            sync_status: SyncStatus::default(),
            toasts: widget::toaster::Toasts::new(ApplicationAction::CloseToast),
            operation_progress: None,
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

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![app_menu::menu_bar(
            &self.key_binds,
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

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, ApplicationAction>> {
        if !self.core.window.show_context {
            return None;
        }
        match self.context_page {
            ContextPage::About => Some(
                context_drawer::about(
                    &self.about,
                    |url| ApplicationAction::OpenExternalUrl(url.to_string()),
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
                    add_account(self.context_account.clone().unwrap()),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::EditAccountForm => Some(
                context_drawer::context_drawer(
                    edit_account(self.context_account.clone().unwrap()),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::NewBookmarkForm => Some(
                context_drawer::context_drawer(
                    new_bookmark(
                        self.context_bookmark.clone().unwrap(),
                        &self.context_bookmark_notes,
                        &self.context_bookmark_description,
                        &self.context_accounts_list,
                        self.context_selected_account_index,
                    ),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::EditBookmarkForm => Some(
                context_drawer::context_drawer(
                    edit_bookmark(
                        self.context_bookmark.clone().unwrap(),
                        &self.context_bookmark_notes,
                        &self.context_bookmark_description,
                        self.context_account.as_ref().unwrap(),
                    ),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
            ContextPage::ViewBookmarkNotes => Some(
                context_drawer::context_drawer(
                    view_notes(&self.context_bookmark_notes),
                    ApplicationAction::ContextClose,
                )
                .title(self.context_page.title()),
            ),
        }
    }

    fn dialog(&self) -> Option<Element<'_, ApplicationAction>> {
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
            DialogPage::ExportBookmarks(accounts, selected, path) => {
                let spacing = cosmic::theme::active().cosmic().spacing;
                let mut body_column = widget::column::with_capacity(3).spacing(spacing.space_s);

                body_column = body_column.push(widget::text::body(fl!("export-bookmarks-body")));

                let mut accounts_list =
                    widget::column::with_capacity(accounts.len()).spacing(spacing.space_xxs);

                for (idx, account) in accounts.iter().enumerate() {
                    let is_selected = selected.get(idx).copied().unwrap_or(false);
                    let checkbox = widget::checkbox(&account.display_name, is_selected).on_toggle(
                        move |checked| {
                            let mut new_selected = selected.clone();
                            if idx < new_selected.len() {
                                new_selected[idx] = checked;
                            }
                            ApplicationAction::ExportBookmarksSelectAccounts(new_selected)
                        },
                    );
                    accounts_list = accounts_list.push(checkbox);
                }

                let accounts_container = widget::container(
                    widget::container(accounts_list)
                        .padding([spacing.space_xs, spacing.space_s])
                        .width(Length::Fill),
                )
                .padding([spacing.space_xxs, 0])
                .width(Length::Fill)
                .class(theme::Container::Background);

                body_column = body_column.push(
                    widget::column::with_capacity(2)
                        .spacing(spacing.space_xxs)
                        .push(widget::text::caption(fl!("select-accounts")))
                        .push(accounts_container),
                );

                let path_container = widget::container(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .align_y(cosmic::iced::Alignment::Center)
                        .push(
                            widget::container(widget::text::body(if let Some(p) = path {
                                p.to_string_lossy().to_string()
                            } else {
                                fl!("no-file-selected")
                            }))
                            .width(Length::Fill)
                            .padding([spacing.space_xxs, spacing.space_xs])
                            .class(theme::Container::Background),
                        )
                        .push(
                            widget::button::standard(fl!("browse"))
                                .on_press(ApplicationAction::SelectExportPath),
                        ),
                )
                .width(Length::Fill);

                body_column = body_column.push(path_container);

                let has_selection = selected.iter().any(|&s| s);
                let has_path = path.is_some();
                let selected_accounts: Vec<Account> = accounts
                    .iter()
                    .zip(selected.iter())
                    .filter(|(_, &sel)| sel)
                    .map(|(acc, _)| acc.clone())
                    .collect();

                widget::dialog()
                    .title(fl!("export-bookmarks"))
                    .icon(icon::from_name("document-save-symbolic").size(58))
                    .control(body_column)
                    .primary_action(if has_selection && has_path {
                        widget::button::suggested(fl!("export"))
                            .on_press(ApplicationAction::PerformExportBookmarks(selected_accounts))
                    } else {
                        widget::button::suggested(fl!("export"))
                    })
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(ApplicationAction::DialogCancel),
                    )
            }
            DialogPage::ImportBookmarks(accounts, selected_idx, path) => {
                let spacing = cosmic::theme::active().cosmic().spacing;
                let mut body_column = widget::column::with_capacity(3).spacing(spacing.space_s);

                body_column = body_column.push(widget::text::body(fl!("import-bookmarks-body")));

                let account_dropdown = widget::row::with_capacity(2)
                    .spacing(spacing.space_xs)
                    .align_y(cosmic::iced::Alignment::Center)
                    .push(
                        widget::container(widget::text::body(fl!("account")))
                            .padding([spacing.space_xxs, spacing.space_xs])
                            .align_y(cosmic::iced::alignment::Vertical::Center)
                            .height(Length::Shrink),
                    )
                    .push({
                        let account_names: Vec<String> = accounts
                            .iter()
                            .map(|acc| acc.display_name.clone())
                            .collect();
                        widget::container(
                            widget::dropdown(account_names, Some(*selected_idx), move |idx| {
                                ApplicationAction::ImportBookmarksSelectAccount(idx)
                            })
                            .width(Length::Fixed(150.0)),
                        )
                        .class(theme::Container::Background)
                    });

                body_column = body_column.push(account_dropdown);

                let path_container = widget::container(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .align_y(cosmic::iced::Alignment::Center)
                        .push(
                            widget::container(widget::text::body(if let Some(p) = path {
                                p.to_string_lossy().to_string()
                            } else {
                                fl!("no-file-selected")
                            }))
                            .width(Length::Fill)
                            .padding([spacing.space_xxs, spacing.space_xs])
                            .class(theme::Container::Background),
                        )
                        .push(
                            widget::button::standard(fl!("browse"))
                                .on_press(ApplicationAction::SelectImportPath),
                        ),
                )
                .width(Length::Fill);

                body_column = body_column.push(path_container);

                let has_path = path.is_some();

                widget::dialog()
                    .title(fl!("import-bookmarks"))
                    .icon(icon::from_name("document-open-symbolic").size(58))
                    .control(body_column)
                    .primary_action(if has_path {
                        widget::button::suggested(fl!("import")).on_press(
                            ApplicationAction::PerformImportBookmarks(
                                accounts[*selected_idx].clone(),
                            ),
                        )
                    } else {
                        widget::button::suggested(fl!("import"))
                    })
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(ApplicationAction::DialogCancel),
                    )
            }
        };

        Some(dialog.into())
    }

    fn view(&self) -> Element<'_, Self::Message> {
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

                self.context_accounts_list = block_on(async {
                    db::SqliteDatabase::select_accounts(
                        self.bookmarks_cursor.database.as_mut().unwrap(),
                    )
                    .await
                });

                if self.context_selected_account_index >= self.context_accounts_list.len() {
                    self.context_selected_account_index = if self.context_accounts_list.is_empty() {
                        0
                    } else {
                        self.context_accounts_list.len() - 1
                    };
                }

                // NOTE: (vkhitrin) For future reference if we need to perform some validations for
                //       local provider.
                if let Some(ref mut database) = &mut self.accounts_cursor.database {
                    for account in &mut self.accounts_view.accounts {
                        if account.is_local_provider() {
                            let current_version =
                                provider::get_provider_version(account.provider(), None);
                            if account.provider_version != current_version {
                                account.provider_version = current_version;
                                let account_clone = account.clone();
                                block_on(async {
                                    db::SqliteDatabase::update_account(database, &account_clone)
                                        .await;
                                });
                            }
                        }
                    }
                }

                self.accounts_view
                    .accounts
                    .sort_by_key(|account| match account.provider() {
                        Provider::Cosmicding => 0,
                        Provider::Linkding => 1,
                    });
                // FIXME: (vkhitrin) If an account is deleted during refresh (should not be
                //        possible without interacting with the database manually, a crash will
                //        occur if an account context window is open.
                if self.context_account.is_some() && !self.accounts_view.accounts.is_empty() {
                    self.context_account = self
                        .accounts_view
                        .accounts
                        .clone()
                        .iter()
                        .find(|account| account.id == self.context_account.as_ref().unwrap().id)
                        .cloned();
                }
            }
            ApplicationAction::AddAccountForm => {
                self.context_account = Some(Account::new(
                    String::new(),
                    String::new(),
                    String::new(),
                    Provider::Linkding,
                ));
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::AddAccountForm,
                )));
            }
            ApplicationAction::EditAccountForm(account) => {
                self.context_account = Some(account.clone());
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
                    self.accounts_view
                        .accounts
                        .sort_by_key(|account| match account.provider() {
                            Provider::Cosmicding => 0,
                            Provider::Linkding => 1,
                        });
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
                        commands.push(Task::perform(
                            provider::fetch_account_details(account),
                            message,
                        ));
                    }
                }
            }
            ApplicationAction::DoneAddAccount(mut account, api_response) => {
                if let Some(response) = api_response {
                    if response.error.is_none() {
                        account.provider_version =
                            provider::get_provider_version(account.provider(), Some(&response));

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
                    commands.push(Task::perform(
                        provider::fetch_account_details(account),
                        message,
                    ));
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
                            account.provider_version =
                                provider::get_provider_version(account.provider(), Some(&response));
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
                    } else {
                        if account.is_local_provider() {
                            account.provider_version =
                                provider::get_provider_version(account.provider(), None);
                        }

                        if current_account != account {
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
            }
            ApplicationAction::BookmarksView(message) => {
                commands.push(self.bookmarks_view.update(message));
            }
            ApplicationAction::StartRefreshBookmarksForAllAccounts => {
                if !self.accounts_view.accounts.is_empty() {
                    if let ApplicationState::Refreshing = self.state {
                    } else {
                        let enabled_accounts: Vec<Account> = self
                            .accounts_view
                            .accounts
                            .iter()
                            .filter(|a| a.enabled && !a.is_local_provider())
                            .cloned()
                            .collect();

                        if enabled_accounts.is_empty() {
                            self.state = ApplicationState::NoEnabledRemoteAccounts;
                            commands.push(self.update(ApplicationAction::LoadBookmarks));
                        } else {
                            self.state = ApplicationState::Refreshing;
                            let total_accounts = enabled_accounts.len();
                            self.operation_progress = Some(OperationProgress {
                                operation_id: 0,
                                total: total_accounts,
                                current: 0,
                                operation_label: fl!("refreshing-accounts"),
                                cancellable: false,
                            });

                            let first_account = enabled_accounts[0].clone();
                            let remaining_accounts = enabled_accounts[1..].to_vec();

                            let message = move |response: DetailedResponse| {
                                cosmic::Action::App(ApplicationAction::DoneRefreshSingleAccount(
                                    response,
                                    remaining_accounts.clone(),
                                ))
                            };

                            commands.push(Task::perform(
                                provider::fetch_bookmarks_for_single_account(first_account),
                                message,
                            ));
                        }
                    }
                }
            }
            ApplicationAction::DoneRefreshSingleAccount(response, remaining_accounts) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if !response.successful {
                        log::error!(
                            "Failed to refresh account: {}",
                            response.account.display_name
                        );
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

                if let Some(ref mut progress) = self.operation_progress {
                    progress.current += 1;

                    if remaining_accounts.is_empty() {
                        self.operation_progress = None;
                        commands.push(self.update(ApplicationAction::LoadAccounts));
                        commands.push(self.update(ApplicationAction::LoadBookmarks));
                        self.state = ApplicationState::Ready;
                        self.sync_status = SyncStatus::Successful;
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!("refreshed-bookmarks")))
                                .map(cosmic::Action::App),
                        );
                    } else {
                        let next_account = remaining_accounts[0].clone();
                        let remaining = remaining_accounts[1..].to_vec();

                        let message = move |response: DetailedResponse| {
                            cosmic::Action::App(ApplicationAction::DoneRefreshSingleAccount(
                                response,
                                remaining.clone(),
                            ))
                        };

                        commands.push(Task::perform(
                            provider::fetch_bookmarks_for_single_account(next_account),
                            message,
                        ));
                    }
                }
            }

            ApplicationAction::StartRefreshBookmarksForAccount(account) => {
                if let ApplicationState::Refreshing = self.state {
                } else if account.enabled {
                    self.state = ApplicationState::Refreshing;

                    self.operation_progress = Some(OperationProgress {
                        operation_id: 0,
                        total: 1,
                        current: 0,
                        operation_label: fl!("refreshing-accounts"),
                        cancellable: false,
                    });

                    let cloned_account = account.clone();
                    let message = move |response: DetailedResponse| {
                        cosmic::Action::App(ApplicationAction::DoneRefreshSingleAccount(
                            response,
                            Vec::new(),
                        ))
                    };

                    commands.push(self.update(ApplicationAction::StartRefreshAccountProfile(
                        account.clone(),
                    )));
                    commands.push(Task::perform(
                        provider::fetch_bookmarks_for_single_account(cloned_account),
                        message,
                    ));
                } else {
                    let has_enabled_remote_accounts = self
                        .accounts_view
                        .accounts
                        .iter()
                        .any(|a| a.enabled && !a.is_local_provider());
                    if !has_enabled_remote_accounts {
                        self.state = ApplicationState::NoEnabledRemoteAccounts;
                    }
                    commands.push(self.update(ApplicationAction::LoadBookmarks));
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
                    commands.push(Task::perform(
                        provider::fetch_account_details(account),
                        message,
                    ));
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
                if matches!(
                    self.state,
                    ApplicationState::Ready | ApplicationState::NoEnabledRemoteAccounts
                ) {
                    self.context_accounts_list =
                        tokio::runtime::Runtime::new().unwrap().block_on(async {
                            db::SqliteDatabase::select_accounts(
                                self.bookmarks_cursor.database.as_mut().unwrap(),
                            )
                            .await
                        });
                    self.context_accounts_list.sort_by_key(|account| {
                        if account.is_local_provider() {
                            1
                        } else {
                            0
                        }
                    });
                    if !self.accounts_view.accounts.is_empty() {
                        self.context_bookmark = Some(Bookmark::new(
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
                        if !self.context_bookmark_notes.text().is_empty() {
                            self.context_bookmark_notes = widget::text_editor::Content::new();
                        }
                        if !self.context_bookmark_description.text().is_empty() {
                            self.context_bookmark_description = widget::text_editor::Content::new();
                        }
                        commands.push(self.update(ApplicationAction::ToggleContextPage(
                            ContextPage::NewBookmarkForm,
                        )));
                    }
                }
            }
            ApplicationAction::SetAccountDisplayName(name) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.display_name = name;
                }
            }
            ApplicationAction::SetAccountInstance(url) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.instance = url;
                }
            }
            ApplicationAction::SetAccountAPIKey(key) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.api_token = key;
                }
            }
            ApplicationAction::SetAccountStatus(enabled) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.enabled = enabled;
                }
            }
            ApplicationAction::SetAccountTrustInvalidCertificates(trust) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.trust_invalid_certs = trust;
                }
            }
            ApplicationAction::SetAccountProvider(provider) => {
                if let Some(ref mut account) = &mut self.context_account {
                    account.set_provider(provider);
                }
            }
            ApplicationAction::AddBookmarkFormAccountIndex(idx) => {
                self.context_selected_account_index = idx;
            }
            ApplicationAction::SetBookmarkURL(url) => {
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.url = url;
                }
            }
            ApplicationAction::SetBookmarkTitle(title) => {
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.title = title;
                }
            }
            ApplicationAction::InputBookmarkNotes(action) => {
                self.context_bookmark_notes.perform(action);
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.notes = self.context_bookmark_notes.text().trim().to_string();
                }
            }
            ApplicationAction::InputBookmarkDescription(action) => {
                self.context_bookmark_description.perform(action);
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.description =
                        self.context_bookmark_description.text().trim().to_string();
                }
            }
            ApplicationAction::SetBookmarkTags(tags_string) => {
                let tags: Vec<String> = if tags_string.is_empty() {
                    Vec::new()
                } else {
                    tags_string.split(' ').map(|s| s.to_string()).collect()
                };
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.tag_names = tags;
                }
            }
            ApplicationAction::SetBookmarkArchived(archived) => {
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.is_archived = archived;
                }
            }
            ApplicationAction::SetBookmarkUnread(unread) => {
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.unread = unread;
                }
            }
            ApplicationAction::SetBookmarkShared(shared) => {
                if let Some(ref mut bookmark) = &mut self.context_bookmark {
                    bookmark.shared = shared;
                }
            }
            // NOTE: (vkhitrin) during creation, linkding doesn't populate 'favicon_url'.
            //       In order to display the new favicon, users are required to wait a
            //       bit, and then perform a manual refresh.
            ApplicationAction::StartAddBookmark(
                account,
                mut bookmark,
                import_context,
                remaining_bookmarks,
            ) => {
                if let Some(ctx) = &import_context {
                    let is_cancelled = match &self.operation_progress {
                        None => true,
                        Some(progress) => progress.operation_id != ctx.import_id,
                    };

                    if is_cancelled {
                        log::debug!(
                            "Skipping bookmark {} - import {} was cancelled",
                            bookmark.url,
                            ctx.import_id
                        );
                        commands.push(self.update(ApplicationAction::DoneImportBookmarks(0)));
                        return Task::batch(commands);
                    }
                }

                bookmark.tag_names = bookmark
                    .tag_names
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();

                let cloned_acc = account.clone();
                let cloned_context = import_context.clone();
                let cloned_remaining = remaining_bookmarks.clone();
                let message = move |api_response: Option<BookmarkCheckDetailsResponse>| {
                    cosmic::Action::App(ApplicationAction::DoneAddBookmark(
                        cloned_acc.clone(),
                        api_response,
                        cloned_context.clone(),
                        cloned_remaining.clone(),
                    ))
                };
                commands.push(Task::perform(
                    provider::populate_bookmark(account, bookmark, true, true),
                    message,
                ));
                self.core.window.show_context = false;
            }
            ApplicationAction::DoneAddBookmark(
                account,
                api_response,
                import_context,
                mut remaining_bookmarks,
            ) => {
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    if let Some(response) = api_response {
                        if response.error.is_none() {
                            if let Some(mut bkmrk) = response.bookmark {
                                bkmrk.is_owner = Some(true);
                                if response.is_new {
                                    block_on(async {
                                        db::SqliteDatabase::add_bookmark(database, &bkmrk).await;
                                    });
                                    // Only show toast if not importing
                                    if import_context.is_none() {
                                        commands.push(
                                            self.toasts
                                                .push(widget::toaster::Toast::new(fl!(
                                                    "added-bookmark-to-account",
                                                    bkmrk = bkmrk.url.clone(),
                                                    acc = account.display_name.clone()
                                                )))
                                                .map(cosmic::Action::App),
                                        );
                                    }
                                } else {
                                    block_on(async {
                                        db::SqliteDatabase::update_bookmark(
                                            database, &bkmrk, &bkmrk,
                                        )
                                        .await;
                                    });
                                    // Only show toast if not importing
                                    if import_context.is_none() {
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
                                }
                                commands.push(self.update(ApplicationAction::LoadBookmarks));

                                if !remaining_bookmarks.is_empty() && import_context.is_some() {
                                    if let Some(ref mut progress) = self.operation_progress {
                                        progress.current += 1;
                                    }

                                    let (next_bookmark, next_context) =
                                        remaining_bookmarks.remove(0);
                                    commands.push(self.update(
                                        ApplicationAction::StartAddBookmark(
                                            account.clone(),
                                            next_bookmark,
                                            Some(next_context),
                                            remaining_bookmarks.clone(),
                                        ),
                                    ));
                                } else if let Some(ctx) = &import_context {
                                    self.operation_progress = None;
                                    commands.push(self.update(
                                        ApplicationAction::DoneImportBookmarks(ctx.total_count),
                                    ));
                                }
                            }
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(
                                        response.error.clone().unwrap(),
                                    ))
                                    .map(cosmic::Action::App),
                            );

                            // Cancel remaining imports
                            if let Some(ctx) = &import_context {
                                let remaining_count = remaining_bookmarks.len();
                                log::error!(
                                    "Bookmark import failed: {}. Cancelling remaining {} imports.",
                                    response.error.unwrap(),
                                    remaining_count
                                );
                                commands.push(self.update(
                                    ApplicationAction::CancelImportBookmarks(ctx.import_id),
                                ));
                            }
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
                        provider::remove_bookmark(account, bookmark),
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
                self.context_bookmark = Some(bookmark.clone());
                self.context_bookmark_notes = widget::text_editor::Content::with_text(
                    &self.context_bookmark.as_ref().unwrap().notes,
                );
                self.context_bookmark_description = widget::text_editor::Content::with_text(
                    &self.context_bookmark.as_ref().unwrap().description,
                );
                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account: Account = block_on(async {
                        db::SqliteDatabase::select_single_account(database, account_id).await
                    });
                    self.context_account = Some(account);
                }
                commands.push(self.update(ApplicationAction::ToggleContextPage(
                    ContextPage::EditBookmarkForm,
                )));
            }
            ApplicationAction::StartEditBookmark(account, mut bookmark) => {
                bookmark.tag_names = bookmark
                    .tag_names
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();

                let cloned_acc = account.clone();
                let message = move |api_response: Option<BookmarkCheckDetailsResponse>| {
                    cosmic::Action::App(ApplicationAction::DoneEditBookmark(
                        cloned_acc.clone(),
                        api_response,
                    ))
                };
                commands.push(Task::perform(
                    provider::populate_bookmark(account, bookmark, false, false),
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
                self.context_bookmark_notes =
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
                        DialogPage::ExportBookmarks(_, _, _)
                        | DialogPage::ImportBookmarks(_, _, _) => {}
                    }
                }
                commands.push(self.update(ApplicationAction::LoadAccounts));
            }
            ApplicationAction::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            ApplicationAction::StartExportBookmarks => {
                let enabled_accounts: Vec<Account> = self
                    .accounts_view
                    .accounts
                    .iter()
                    .filter(|acc| acc.enabled)
                    .cloned()
                    .collect();

                if !enabled_accounts.is_empty() {
                    let selected = vec![false; enabled_accounts.len()];
                    if self.dialog_pages.pop_front().is_none() {
                        self.dialog_pages.push_back(DialogPage::ExportBookmarks(
                            enabled_accounts,
                            selected,
                            None,
                        ));
                    }
                }
            }
            ApplicationAction::ExportBookmarksSelectAccounts(selected) => {
                if let Some(DialogPage::ExportBookmarks(accounts, _, path)) =
                    self.dialog_pages.front()
                {
                    self.dialog_pages[0] =
                        DialogPage::ExportBookmarks(accounts.clone(), selected, path.clone());
                }
            }
            ApplicationAction::StartImportBookmarks => {
                let enabled_accounts: Vec<Account> = self
                    .accounts_view
                    .accounts
                    .iter()
                    .filter(|acc| acc.enabled)
                    .cloned()
                    .collect();

                if !enabled_accounts.is_empty() && self.dialog_pages.pop_front().is_none() {
                    self.dialog_pages.push_back(DialogPage::ImportBookmarks(
                        enabled_accounts,
                        0,
                        None,
                    ));
                }
            }
            ApplicationAction::ImportBookmarksSelectAccount(idx) => {
                if let Some(DialogPage::ImportBookmarks(accounts, _, path)) =
                    self.dialog_pages.front()
                {
                    self.dialog_pages[0] =
                        DialogPage::ImportBookmarks(accounts.clone(), idx, path.clone());
                }
            }
            ApplicationAction::SelectExportPath => {
                commands.push(Task::perform(
                    async { open_save_file_dialog("cosmicding_bookmarks_export.html").await },
                    |path| cosmic::Action::App(ApplicationAction::SetExportPath(path)),
                ));
            }
            ApplicationAction::SelectImportPath => {
                commands.push(Task::perform(async { open_file_dialog().await }, |path| {
                    cosmic::Action::App(ApplicationAction::SetImportPath(path))
                }));
            }
            ApplicationAction::SetExportPath(path) => {
                if let Some(DialogPage::ExportBookmarks(accounts, selected, _)) =
                    self.dialog_pages.front()
                {
                    self.dialog_pages[0] =
                        DialogPage::ExportBookmarks(accounts.clone(), selected.clone(), path);
                }
            }
            ApplicationAction::SetImportPath(path) => {
                if let Some(DialogPage::ImportBookmarks(accounts, idx, _)) =
                    self.dialog_pages.front()
                {
                    self.dialog_pages[0] =
                        DialogPage::ImportBookmarks(accounts.clone(), *idx, path);
                }
            }
            ApplicationAction::PerformExportBookmarks(accounts) => {
                let export_path_from_dialog = if let Some(DialogPage::ExportBookmarks(_, _, path)) =
                    self.dialog_pages.front()
                {
                    path.clone()
                } else {
                    None
                };

                self.dialog_pages.pop_front();

                if let Some(ref mut database) = &mut self.bookmarks_cursor.database {
                    let account_ids: Vec<i64> = accounts.iter().filter_map(|acc| acc.id).collect();

                    block_on(async {
                        let total_count = database.count_bookmarks_entries().await;

                        let mut all_bookmarks: Vec<Bookmark> = Vec::new();
                        let limit: u8 = 255;
                        let mut offset: usize = 0;

                        while offset < total_count {
                            let bookmarks = database
                                .select_bookmarks_with_limit(
                                    limit,
                                    offset,
                                    self.bookmarks_cursor.sort_option,
                                )
                                .await;

                            if bookmarks.is_empty() {
                                break;
                            }

                            all_bookmarks.extend(bookmarks);
                            offset += limit as usize;
                        }

                        let filtered_bookmarks: Vec<Bookmark> = all_bookmarks
                            .into_iter()
                            .filter(|bm| {
                                if let Some(user_account_id) = bm.user_account_id {
                                    account_ids.contains(&user_account_id)
                                } else {
                                    false
                                }
                            })
                            .collect();

                        let bookmark_count = filtered_bookmarks.len();

                        let html_content = bookmark_parser::netscape::BookmarkIO::generate(
                            &filtered_bookmarks,
                            bookmark_parser::netscape::BookmarkFormat::Netscape,
                        );

                        if let Some(export_path) = export_path_from_dialog {
                            match std::fs::write(&export_path, html_content) {
                                Ok(()) => {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "export-bookmarks-success",
                                                count = bookmark_count,
                                                path = export_path.display().to_string()
                                            )))
                                            .map(cosmic::Action::App),
                                    );
                                }
                                Err(e) => {
                                    commands.push(
                                        self.toasts
                                            .push(widget::toaster::Toast::new(fl!(
                                                "export-bookmarks-error",
                                                error = e.to_string()
                                            )))
                                            .map(cosmic::Action::App),
                                    );
                                }
                            }
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!(
                                        "export-bookmarks-no-path"
                                    )))
                                    .map(cosmic::Action::App),
                            );
                        }
                    });
                }
            }
            ApplicationAction::PerformImportBookmarks(account) => {
                let import_path_from_dialog = if let Some(DialogPage::ImportBookmarks(_, _, path)) =
                    self.dialog_pages.front()
                {
                    path.clone()
                } else {
                    None
                };

                self.dialog_pages.pop_front();

                self.state = ApplicationState::Refreshing;

                if let Some(import_path) = import_path_from_dialog {
                    if import_path.exists() {
                        match std::fs::read_to_string(&import_path) {
                            Ok(html_content) => {
                                match bookmark_parser::netscape::BookmarkIO::parse(
                                    &html_content,
                                    bookmark_parser::netscape::BookmarkFormat::Netscape,
                                ) {
                                    Ok(bookmarks) => {
                                        let import_count = bookmarks.len();
                                        if import_count == 0 {
                                            commands.push(
                                                self.toasts
                                                    .push(widget::toaster::Toast::new(fl!(
                                                        "import-bookmarks-error",
                                                        error = "No bookmarks found"
                                                    )))
                                                    .map(cosmic::Action::App),
                                            );
                                            commands.push(
                                                self.update(
                                                    ApplicationAction::DoneImportBookmarks(0),
                                                ),
                                            );
                                        } else {
                                            let import_id = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_nanos()
                                                as u64;

                                            let mut bookmarks_with_context: Vec<(
                                                Bookmark,
                                                ImportAction,
                                            )> = bookmarks
                                                .into_iter()
                                                .enumerate()
                                                .map(|(index, bookmark)| {
                                                    let import_context = ImportAction {
                                                        import_id,
                                                        total_count: import_count,
                                                        current_index: index,
                                                    };
                                                    (bookmark, import_context)
                                                })
                                                .collect();

                                            let (first_bookmark, first_context) =
                                                bookmarks_with_context.remove(0);

                                            self.operation_progress = Some(OperationProgress {
                                                operation_id: import_id,
                                                total: import_count,
                                                current: 0,
                                                operation_label: fl!("importing-bookmarks"),
                                                cancellable: true,
                                            });

                                            commands.push(self.update(
                                                ApplicationAction::StartAddBookmark(
                                                    account.clone(),
                                                    first_bookmark,
                                                    Some(first_context),
                                                    bookmarks_with_context,
                                                ),
                                            ));

                                            commands.push(
                                                self.toasts
                                                    .push(widget::toaster::Toast::new(fl!(
                                                        "import-bookmarks-started",
                                                        count = import_count
                                                    )))
                                                    .map(cosmic::Action::App),
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        commands.push(
                                            self.toasts
                                                .push(widget::toaster::Toast::new(fl!(
                                                    "import-bookmarks-error",
                                                    error = e.to_string()
                                                )))
                                                .map(cosmic::Action::App),
                                        );
                                        commands.push(
                                            self.update(ApplicationAction::DoneImportBookmarks(0)),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                commands.push(
                                    self.toasts
                                        .push(widget::toaster::Toast::new(fl!(
                                            "import-bookmarks-error",
                                            error = e.to_string()
                                        )))
                                        .map(cosmic::Action::App),
                                );
                                commands
                                    .push(self.update(ApplicationAction::DoneImportBookmarks(0)));
                            }
                        }
                    } else {
                        commands.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!(
                                    "import-bookmarks-file-not-found",
                                    path = import_path.display().to_string()
                                )))
                                .map(cosmic::Action::App),
                        );
                        commands.push(self.update(ApplicationAction::DoneImportBookmarks(0)));
                    }
                } else {
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("import-bookmarks-no-path")))
                            .map(cosmic::Action::App),
                    );
                    commands.push(self.update(ApplicationAction::DoneImportBookmarks(0)));
                }
            }
            ApplicationAction::CancelImportBookmarks(import_id) => {
                log::info!("Import {} cancelled", import_id);

                self.operation_progress = None;

                commands.push(self.update(ApplicationAction::DoneImportBookmarks(0)));
            }
            ApplicationAction::DoneImportBookmarks(count) => {
                self.state = ApplicationState::Ready;
                self.operation_progress = None;

                if count > 0 {
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "import-bookmarks-finished",
                                count = count
                            )))
                            .map(cosmic::Action::App),
                    );
                }
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
                                        provider::fetch_bookmark_favicon(favicon_url_clone.clone()),
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
                    if !account.is_local_provider() {
                        commands.push(
                            self.update(ApplicationAction::StartRefreshAccountProfile(account)),
                        );
                    }
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
                if matches!(
                    self.state,
                    ApplicationState::Ready | ApplicationState::NoEnabledRemoteAccounts
                ) {
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
    fn settings(&self) -> Element<'_, ApplicationAction> {
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
