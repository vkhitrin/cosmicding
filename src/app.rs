use crate::config::{AppTheme, Config, CONFIG_VERSION};
use crate::db::{self, SqliteDatabase};
use crate::fl;
use crate::http::{self};
use crate::key_binds::key_binds;
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;
use crate::nav::NavPage;
use crate::pages::accounts::{add_account, edit_account, AccountsMessage, AccountsView};
use crate::pages::bookmarks::{
    edit_bookmark, new_bookmark, view_notes, BookmarksMessage, BookmarksView,
};
use cosmic::app::{Command, Core};
use cosmic::cosmic_config::{self, CosmicConfigEntry, Update};
use cosmic::cosmic_theme::{self, ThemeMode};
use cosmic::iced::{
    event,
    keyboard::{Event as KeyEvent, Key, Modifiers},
    Alignment, Event, Length, Subscription,
};
use cosmic::widget::menu::action::MenuAction as _MenuAction;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{theme, Application, ApplicationExt, Element};
use futures::executor::block_on;
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};

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
    config: Config,
    config_handler: Option<cosmic_config::Config>,
    modifiers: Modifiers,
    app_themes: Vec<String>,
    db: SqliteDatabase,
    pub accounts_view: AccountsView,
    pub bookmarks_view: BookmarksView,
    placeholder_account: Option<Account>,
    placeholder_bookmark: Option<Bookmark>,
    placeholder_selected_account_index: usize,
    toasts: widget::toaster::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AccountsView(AccountsMessage),
    AddAccount,
    AddBookmark(Account, Bookmark),
    AddBookmarkForm,
    AddBookmarkFormAccountIndex(usize),
    AppTheme(AppTheme),
    BookmarksView(BookmarksMessage),
    CloseToast(widget::ToastId),
    CompleteAddAccount(Account),
    CompleteRemoveDialog(Account, Option<Bookmark>),
    DialogCancel,
    DialogUpdate(DialogPage),
    EditAccount(Account),
    EditBookmark(Account, Bookmark),
    FetchAccounts,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    OpenAccountsPage,
    OpenExternalUrl(String),
    OpenRemoveAccountDialog(Account),
    OpenRemoveBookmarkDialog(Account, Bookmark),
    RefreshAllBookmarks,
    RefreshBookmarksForAccount(Account),
    RemoveAccount(Account),
    RemoveBookmark(Account, Bookmark),
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
    SubscriptionChannel,
    SystemThemeModeChange,
    ToggleContextPage(ContextPage),
    UpdateAccount(Account),
    UpdateBookmark(Account, Bookmark),
    UpdateConfig(Config),
    ViewBookmarkNotes(Bookmark),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    RemoveAccount(Account),
    RemoveBookmark(Account, Bookmark),
}

impl Application for Cosmicding {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = "com.vkhitrin.cosmicding";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let db = block_on(async { db::SqliteDatabase::create().await.unwrap() });
        let mut nav = nav_bar::Model::default();
        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        for &nav_page in NavPage::all() {
            let id = nav
                .insert()
                .icon(nav_page.icon())
                .text(nav_page.title())
                .data::<NavPage>(nav_page)
                .id();

            if nav_page == NavPage::default() {
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
            db,
            dialog_pages: VecDeque::new(),
            accounts_view: AccountsView::default(),
            bookmarks_view: BookmarksView::default(),
            placeholder_account: None,
            placeholder_bookmark: None,
            placeholder_selected_account_index: 0,
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let commands = vec![
            app.update_title(),
            app.update(Message::FetchAccounts),
            app.update(Message::RefreshAllBookmarks),
        ];

        (app, Command::batch(commands))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![crate::menu::menu_bar(&self.key_binds)]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_escape(&mut self) -> Command<Message> {
        if self.dialog_pages.pop_front().is_some() {
            return Command::none();
        }

        self.core.window.show_context = false;

        Command::none()
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
                &self.accounts_view.accounts,
                self.placeholder_selected_account_index,
            ),
            ContextPage::EditBookmarkForm => edit_bookmark(
                self.placeholder_bookmark.clone().unwrap(),
                &self.accounts_view.accounts,
            ),
            ContextPage::ViewBookmarkNotes => {
                view_notes(self.placeholder_bookmark.clone().unwrap())
            }
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::RemoveAccount(account) => {
                widget::dialog(fl!("remove") + " " + { &account.display_name })
                    .icon(icon::from_name("dialog-warning-symbolic").size(58).icon())
                    .body(fl!("remove-account-confirm"))
                    .primary_action(
                        widget::button::destructive(fl!("yes")).on_press_maybe(Some(
                            Message::CompleteRemoveDialog(account.clone(), None),
                        )),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
            DialogPage::RemoveBookmark(account, bookmark) => {
                widget::dialog(fl!("remove") + " " + { &bookmark.title })
                    .icon(icon::from_name("dialog-warning-symbolic").size(58).icon())
                    .body(fl!("remove-bookmark-confirm"))
                    .primary_action(widget::button::destructive(fl!("yes")).on_press_maybe(Some(
                        Message::CompleteRemoveDialog(account.clone(), Some(bookmark.clone())),
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
        let nav_page = self.nav.data::<NavPage>(entity).unwrap_or_default();

        widget::column::with_children(vec![
            (widget::toaster(&self.toasts, widget::horizontal_space(Length::Fill)).into()),
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
        .align_items(Alignment::Center)
        .into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status| match event {
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
                        "errors loading config {:?}: {:?}",
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
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange
            }),
        ];

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut commands = vec![];
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        if let Err(err) =
                            paste::paste! { self.config.[<set_ $name>](config_handler, $value) }
                        {
                            log::warn!("failed to save config {:?}: {}", stringify!($name), err);
                        }
                    }
                    None => {
                        self.config.$name = $value;
                    }
                }
            };
        }
        match message {
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::SystemThemeModeChange => {
                return self.update_config();
            }
            Message::OpenAccountsPage => {
                let account_page_entity = &self.nav.entity_at(0);
                _ = self.nav.activate(account_page_entity.unwrap());
            }

            Message::SubscriptionChannel => {}

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                self.set_context_title(context_page.title());
            }
            Message::AccountsView(message) => commands.push(
                self.accounts_view
                    .update(message)
                    .map(cosmic::app::Message::App),
            ),
            Message::FetchAccounts => {
                self.accounts_view.accounts =
                    block_on(async { db::SqliteDatabase::fetch_accounts(&mut self.db).await });
                self.bookmarks_view.accounts = self.accounts_view.accounts.clone();
                self.placeholder_account = None;
            }
            Message::AddAccount => {
                self.placeholder_account =
                    Some(Account::new("".to_owned(), "".to_owned(), "".to_owned()));
                commands.push(self.update(Message::ToggleContextPage(ContextPage::AddAccountForm)));
            }
            Message::EditAccount(account) => {
                self.placeholder_account = Some(account.clone());
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::EditAccountForm)));
            }
            Message::RemoveAccount(account) => {
                block_on(async {
                    db::SqliteDatabase::delete_all_bookmarks_of_account(&mut self.db, &account)
                        .await
                });
                block_on(async {
                    db::SqliteDatabase::delete_account(&mut self.db, &account).await
                });
                self.bookmarks_view.bookmarks.retain(|bkmrk| bkmrk.user_account_id != account.id);
                commands.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(fl!(
                            "removed-account",
                            acc = account.display_name
                        )))
                        .map(cosmic::app::Message::App),
                );
            }
            Message::CompleteAddAccount(account) => {
                let mut valid_account = false;
                block_on(async {
                    match http::check_instance(&account).await {
                        Ok(()) => {
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
                        db::SqliteDatabase::create_account(&mut self.db, &account).await
                    });
                    commands.push(self.update(Message::FetchAccounts));
                    commands.push(self.update(Message::RefreshBookmarksForAccount(
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
                self.core.window.show_context = false;
            }
            Message::UpdateAccount(account) => {
                let mut valid_account = false;
                block_on(async {
                    match http::check_instance(&account).await {
                        Ok(()) => {
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
                        db::SqliteDatabase::update_account(&mut self.db, &account).await
                    });
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "updated-account",
                                acc = account.display_name
                            )))
                            .map(cosmic::app::Message::App),
                    );
                    commands.push(self.update(Message::FetchAccounts));
                    commands.push(self.update(Message::RefreshBookmarksForAccount(
                        self.accounts_view.accounts.last().unwrap().clone(),
                    )));
                }
                self.core.window.show_context = false;
                commands.push(self.update(Message::FetchAccounts));
                self.core.window.show_context = false;
            }
            Message::BookmarksView(message) => commands.push(
                self.bookmarks_view
                    .update(message)
                    .map(cosmic::app::Message::App),
            ),
            Message::RefreshAllBookmarks => {
                if !self.accounts_view.accounts.is_empty() {
                    let remote_bookmarks = block_on(async {
                        http::fetch_all_bookmarks_from_accounts(self.accounts_view.accounts.clone())
                            .await
                    });
                    block_on(async {
                        db::SqliteDatabase::cache_all_bookmarks(&mut self.db, remote_bookmarks)
                            .await
                    });
                }
                self.bookmarks_view.bookmarks =
                    block_on(async { db::SqliteDatabase::fetch_bookmarks(&mut self.db).await });
                if !self.bookmarks_view.bookmarks.is_empty() {
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("refreshed-all-bookmarks")))
                            .map(cosmic::app::Message::App),
                    );
                }
            }
            Message::RefreshBookmarksForAccount(account) => {
                let mut remote_bookmarks: Vec<Bookmark> = Vec::new();
                block_on(async {
                    match http::fetch_bookmarks_for_account(&account).await {
                        Ok(new_bookmarks) => {
                            remote_bookmarks.extend(new_bookmarks);
                        }
                        Err(e) => {
                            eprintln!("Error fetching bookmarks: {}", e);
                        }
                    }
                });
                block_on(async {
                    db::SqliteDatabase::cache_bookmarks_for_acount(
                        &mut self.db,
                        &account,
                        remote_bookmarks,
                    )
                    .await
                });
                self.bookmarks_view.bookmarks =
                    block_on(async { db::SqliteDatabase::fetch_bookmarks(&mut self.db).await });
                if !self.bookmarks_view.bookmarks.is_empty() {
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "refreshed-bookmarks-for-account",
                                acc = account.display_name
                            )))
                            .map(cosmic::app::Message::App),
                    );
                } else {
                    commands.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!(
                                "no-bookmarks-found-for-account",
                                acc = account.display_name
                            )))
                            .map(cosmic::app::Message::App),
                    );
                }
            }
            Message::AddBookmarkForm => {
                self.placeholder_bookmark = Some(Bookmark::new(
                    None,
                    None,
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    false,
                    false,
                    false,
                    Vec::new(),
                    None,
                    None,
                ));
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::NewBookmarkForm)));
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
                            eprintln!("Error adding bookmark: {}", e);
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
                        db::SqliteDatabase::add_bookmark(&mut self.db, &bkmrk).await
                    });
                    self.bookmarks_view.bookmarks.push(bkmrk);
                }
                self.core.window.show_context = false;
            }
            Message::RemoveBookmark(account, bookmark) => {
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
                            eprintln!("Error removing bookmark: {}", e);
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(format!("{e}")))
                                    .map(cosmic::app::Message::App),
                            );
                        }
                    }
                });
                block_on(async {
                    db::SqliteDatabase::delete_bookmark(&mut self.db, &bookmark).await
                });
                self.core.window.show_context = false;
            }
            Message::EditBookmark(account, bookmark) => {
                self.placeholder_account = Some(account.clone());
                self.placeholder_bookmark = Some(bookmark.clone());
                commands
                    .push(self.update(Message::ToggleContextPage(ContextPage::EditBookmarkForm)));
            }
            Message::UpdateBookmark(account, bookmark) => {
                let mut updated_bkmrk: Option<Bookmark> = None;
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
                            eprintln!("Error patching bookmark: {}", e);
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(format!("{e}")))
                                    .map(cosmic::app::Message::App),
                            );
                        }
                    }
                });
                println!("{:?}", bookmark);
                if let Some(bkmrk) = updated_bkmrk {
                    let index = self
                        .bookmarks_view
                        .bookmarks
                        .iter()
                        .position(|x| x.id == bookmark.id)
                        .unwrap();
                    println!("{}", index);
                    block_on(async {
                        db::SqliteDatabase::update_bookmark(&mut self.db, &bookmark, &bkmrk).await
                    });
                    self.bookmarks_view.bookmarks[index] = bkmrk;
                }
                self.core.window.show_context = false;
            }
            Message::SearchBookmarks(query) => {
                if !query.is_empty() {
                    self.bookmarks_view.bookmarks = block_on(async {
                        db::SqliteDatabase::search_bookmarks(&mut self.db, query).await
                    });
                } else {
                    self.bookmarks_view.bookmarks =
                        block_on(async { db::SqliteDatabase::fetch_bookmarks(&mut self.db).await });
                }
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
            Message::OpenRemoveAccountDialog(account) => {
                self.dialog_pages
                    .push_back(DialogPage::RemoveAccount(account));
            }
            Message::OpenRemoveBookmarkDialog(account, bookmark) => {
                self.dialog_pages
                    .push_back(DialogPage::RemoveBookmark(account, bookmark));
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
                        DialogPage::RemoveBookmark(account, bookmark) => {
                            commands.push(self.update(Message::RemoveBookmark(account, bookmark)));
                        }
                    }
                }
                commands.push(self.update(Message::FetchAccounts));
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
        }
        Command::batch(commands)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        self.nav.activate(id);

        self.update_title()
    }
}

impl Cosmicding {
    pub fn about(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        widget::column::with_children(vec![
            widget::text::title3(fl!("app-title")).into(),
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
            widget::text::caption(fl!("pre-release")).into(),
        ])
        .align_items(Alignment::Center)
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        let appearance_section = widget::settings::section().title(fl!("appearance")).add(
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
            )),
        );

        widget::settings::view_column(vec![appearance_section.into()]).into()
    }

    fn update_config(&mut self) -> Command<Message> {
        let theme = self.config.app_theme.theme();
        cosmic::app::command::set_theme(theme)
    }

    pub fn update_title(&mut self) -> Command<Message> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.insert_str(0, " — ");
            window_title.insert_str(0, page);
        }

        self.set_window_title(window_title)
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
    fn title(&self) -> String {
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
    Settings,
}

impl _MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::AddAccount => Message::AddAccount,
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}
