use crate::app::{icons::load_icon, ApplicationState, Message};
use crate::fl;
use crate::models::account::Account;
use crate::models::db_cursor::AccountsPaginationCursor;
use crate::style::button::ButtonStyle;
use chrono::{DateTime, Local};
use cosmic::iced::Length;
use cosmic::iced_widget::tooltip;
use cosmic::{
    app::Task,
    cosmic_theme,
    iced::Alignment,
    theme,
    widget::{self},
    Apply, Element,
};

#[derive(Debug, Clone)]
pub enum AppAccountsMessage {
    AddAccount,
    DecrementPageIndex,
    DeleteAccount(Account),
    EditAccount(Account),
    IncrementPageIndex,
    OpenExternalURL(String),
    RefreshBookmarksForAccount(Account),
}

#[derive(Debug, Clone, Default)]
pub struct PageAccountsView {
    pub accounts: Vec<Account>,
    account_placeholder: Option<Account>,
}

impl PageAccountsView {
    #[allow(clippy::too_many_lines)]
    pub fn view(
        &self,
        app_state: ApplicationState,
        accounts_cursor: &AccountsPaginationCursor,
    ) -> Element<'_, AppAccountsMessage> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts.is_empty() {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::icon(load_icon("contact-new-symbolic"))
                        .size(64)
                        .into(),
                    widget::text::title3(fl!("no-accounts")).into(),
                    widget::button::standard(fl!("add-account"))
                        .on_press(AppAccountsMessage::AddAccount)
                        .into(),
                ])
                .spacing(20)
                .align_x(Alignment::Center),
            )
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill);
            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(container)
                .into()
        } else {
            let mut items = widget::list::list_column()
                .spacing(spacing.space_xxxs)
                .padding([spacing.space_none, spacing.space_xxs]);

            for item in &self.accounts {
                let local_time: DateTime<Local> = DateTime::from(
                    DateTime::from_timestamp(item.last_sync_timestamp, 0).expect(""),
                );

                let refresh_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("refresh"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("refresh")).font_size(12).on_press(
                        AppAccountsMessage::RefreshBookmarksForAccount(item.to_owned()),
                    ),
                };
                let edit_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("edit"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("edit"))
                        .font_size(12)
                        .on_press(AppAccountsMessage::EditAccount(item.to_owned())),
                };
                let remove_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .on_press(AppAccountsMessage::DeleteAccount(item.to_owned())),
                };

                // Mandatory first row - details
                let mut columns = Vec::new();
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .push(widget::icon::icon(load_icon("user-available-symbolic")))
                        .push(widget::text(item.display_name.clone()))
                        .padding([
                            spacing.space_xs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory second row - sync timestamp + status
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::icon::icon(load_icon("accessories-clock-symbolic")))
                        .push(widget::text::body(format!(
                            "{}: {} ({})",
                            fl!("last-sync-time"),
                            local_time.format("%a, %d %b %Y %H:%M:%S"),
                            if item.last_sync_status {
                                fl!("successful")
                            } else {
                                fl!("failed")
                            }
                        )))
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory third row - details
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::icon::icon(load_icon("dialog-information-symbolic")))
                        .push(widget::container(widget::column::with_children(vec![
                            if item.tls {
                                widget::text::body(fl!("tls-enabled")).into()
                            } else {
                                widget::text::body(fl!("tls-disabled")).into()
                            },
                            if item.enable_sharing {
                                widget::text::body(fl!("enabled-sharing")).into()
                            } else {
                                widget::text::body(fl!("disabled-sharing")).into()
                            },
                            if item.enable_public_sharing {
                                widget::text::body(fl!("enabled-public-sharing")).into()
                            } else {
                                widget::text::body(fl!("disabled-public-sharing")).into()
                            },
                        ])))
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory forth row - actions
                let actions_row = widget::row::with_capacity(3)
                    .spacing(spacing.space_xs)
                    .push(refresh_button)
                    .push(edit_button)
                    .push(remove_button)
                    .push(
                        widget::button::link(fl!("open-instance"))
                            .spacing(spacing.space_xxxs)
                            .trailing_icon(true)
                            .icon_size(10)
                            .font_size(12)
                            .on_press(AppAccountsMessage::OpenExternalURL(item.instance.clone()))
                            .tooltip(item.instance.clone()),
                    );
                columns.push(
                    actions_row
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_none,
                        ])
                        .into(),
                );
                let account_container = widget::container(widget::column::with_children(columns));

                items = items.add(account_container);
            }

            let accounts_widget = widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(items)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill);

            let navigation_previous_button = widget::button::standard(fl!("previous"))
                .on_press_maybe(if accounts_cursor.current_page > 1 {
                    Some(AppAccountsMessage::DecrementPageIndex)
                } else {
                    None
                })
                .leading_icon(load_icon("go-previous-symbolic"));
            let navigation_next_button = widget::button::standard(fl!("next"))
                .on_press_maybe(
                    if accounts_cursor.current_page < accounts_cursor.total_pages {
                        Some(AppAccountsMessage::IncrementPageIndex)
                    } else {
                        None
                    },
                )
                .trailing_icon(load_icon("go-next-symbolic"));
            let page_navigation_widget = widget::container(widget::column::with_children(vec![
                widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .push(widget::horizontal_space())
                    .push(navigation_previous_button)
                    .push(widget::text::body(format!(
                        "{}/{}",
                        accounts_cursor.current_page, accounts_cursor.total_pages
                    )))
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_xxs,
                        spacing.space_none,
                        spacing.space_xxxs,
                        spacing.space_none,
                    ])
                    .push(navigation_next_button)
                    .push(widget::horizontal_space())
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into(),
            ]));

            widget::container(
                widget::column::with_children(vec![widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .push(widget::text::title3(fl!(
                        "accounts-with-count",
                        count = accounts_cursor.total_entries.to_string()
                    )))
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ])
                    .push(widget::horizontal_space())
                    .push(
                        widget::button::standard(fl!("add-account"))
                            .on_press(AppAccountsMessage::AddAccount),
                    )
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into()])
                .push(accounts_widget)
                .push(page_navigation_widget),
            )
            .into()
        }
    }
    pub fn update(&mut self, message: AppAccountsMessage) -> Task<Message> {
        let mut commands = Vec::new();
        match message {
            AppAccountsMessage::AddAccount => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(Message::AddAccount)
                }));
            }
            AppAccountsMessage::EditAccount(account) => {
                self.account_placeholder = Some(account.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(Message::EditAccount(account.clone()))
                }));
            }
            AppAccountsMessage::DeleteAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(Message::OpenRemoveAccountDialog(account.clone()))
                }));
            }
            AppAccountsMessage::RefreshBookmarksForAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(Message::StartRefreshBookmarksForAccount(account.clone()))
                }));
            }
            AppAccountsMessage::OpenExternalURL(url) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(Message::OpenExternalUrl(url.clone()))
                }));
            }
            AppAccountsMessage::IncrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(Message::IncrementPageIndex("accounts".to_string()))
                }));
            }
            AppAccountsMessage::DecrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(Message::DecrementPageIndex("accounts".to_string()))
                }));
            }
        }
        Task::batch(commands)
    }
}

pub fn add_account<'a>(account: Account) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let display_name_widget_title = widget::text::body(fl!("display-name"));
    let display_name_widget_text_input = widget::text_input("Name", account.display_name.clone())
        .on_input(Message::SetAccountDisplayName);
    let instance_widget_title = widget::text::body(fl!("instance"));
    let instance_widget_text_input = widget::text_input("Instance", account.instance.clone())
        .on_input(Message::SetAccountInstance);
    let api_key_widget_title = widget::text::body(fl!("api-key"));
    let api_key_widget_text_input = widget::text_input(fl!("token"), account.api_token.clone())
        .on_input(Message::SetAccountAPIKey)
        .password();
    let tls_widget_toggler = widget::toggler(account.tls)
        .on_toggle(Message::SetAccountTLS)
        .spacing(10)
        .label(fl!("tls"));
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::CompleteAddAccount(account)),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("user-available-symbolic")))
                .push(display_name_widget_title)
                .padding([
                    spacing.space_none,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(display_name_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("network-server-symbolic")))
                .push(instance_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(instance_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("system-lock-screen-symbolic")))
                .push(api_key_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(api_key_widget_text_input)
        .push(
            widget::row::with_capacity(1)
                .push(tls_widget_toggler)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_xs,
                    spacing.space_none,
                ]),
        )
        .push(buttons_widget_container)
        .into()
}

#[allow(clippy::too_many_lines)]
pub fn edit_account<'a>(account: Account) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let display_name_widget_title = widget::text::body(fl!("display-name"));
    let display_name_widget_text_input = widget::text_input("Name", account.display_name.clone())
        .on_input(Message::SetAccountDisplayName);
    let instance_widget_title = widget::text::body(fl!("instance"));
    let instance_widget_text_input = widget::text_input("Instance", account.instance.clone())
        .on_input(Message::SetAccountInstance);
    let api_key_widget_title = widget::text::body(fl!("api-key"));
    let api_key_widget_text_input = widget::text_input(fl!("token"), account.api_token.clone())
        .on_input(Message::SetAccountAPIKey)
        .password();
    let tls_widget_toggler = widget::toggler(account.tls)
        .on_toggle(Message::SetAccountTLS)
        .spacing(10)
        .label(fl!("tls"));
    let enable_shared_widget_text = if account.enable_sharing {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("enabled-sharing")))
                .push(widget::icon::icon(load_icon("dialog-information-symbolic")).size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    } else {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("disabled-sharing")))
                .push(widget::icon::icon(load_icon("dialog-information-symbolic")).size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    };
    let enable_public_shared_widget_text = if account.enable_public_sharing {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("enabled-public-sharing")))
                .push(widget::icon::icon(load_icon("dialog-information-symbolic")).size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    } else {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("disabled-public-sharing")))
                .push(widget::icon::icon(load_icon("dialog-information-symbolic")).size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::UpdateAccount(account)),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("user-available-symbolic")))
                .push(display_name_widget_title)
                .padding([
                    spacing.space_none,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(display_name_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("network-server-symbolic")))
                .push(instance_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(instance_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("system-lock-screen-symbolic")))
                .push(api_key_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(api_key_widget_text_input)
        .push(
            widget::row::with_capacity(1)
                .push(tls_widget_toggler)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_xs,
                    spacing.space_none,
                ]),
        )
        .push(enable_shared_widget_text)
        .push(enable_public_shared_widget_text)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}
