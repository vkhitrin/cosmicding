use crate::app::Message;
use crate::fl;
use crate::models::account::Account;
use chrono::{DateTime, Local};
use cosmic::iced::Length;
use cosmic::iced_widget::tooltip;
use cosmic::{
    app::command::Task,
    cosmic_theme,
    iced::{self, Alignment},
    theme,
    widget::{self},
    Apply, Element,
};
use iced::alignment::{Horizontal, Vertical};

#[derive(Debug, Clone)]
pub enum AppAccountsMessage {
    AddAccount,
    EditAccount(Account),
    DeleteAccount(Account),
    RefreshBookmarksForAccount(Account),
    OpenExternalURL(String),
}

#[derive(Debug, Clone, Default)]
pub struct PageAccountsView {
    pub accounts: Vec<Account>,
    account_placeholder: Option<Account>,
}

impl PageAccountsView {
    #[allow(clippy::too_many_lines)]
    pub fn view(&self) -> Element<'_, AppAccountsMessage> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts.is_empty() {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("contact-new-symbolic")
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
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
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

                // Mandatory first row - details
                let mut columns = Vec::new();
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .push(widget::icon::from_name("user-available-symbolic"))
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
                // Mandatory second row - sync status
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::icon::from_name("emblem-synchronizing-symbolic"))
                        .push(widget::text::body(format!(
                            "{}: {}",
                            fl!("last-sync-status"),
                            if item.last_sync_status {
                                fl!("successful")
                            } else {
                                fl!("failed")
                            }
                        )))
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory third row - sync timestamp
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::icon::from_name("accessories-clock-symbolic"))
                        .push(widget::text::body(format!(
                            "{}: {}",
                            fl!("last-sync-time"),
                            local_time.to_rfc2822()
                        )))
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory fourth row - details
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::icon::from_name("dialog-information-symbolic"))
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
                // Mandatory fifth row - actions
                let actions_row = widget::row::with_capacity(3)
                    .spacing(spacing.space_xs)
                    .push(widget::button::link(fl!("refresh")).font_size(12).on_press(
                        AppAccountsMessage::RefreshBookmarksForAccount(item.to_owned()),
                    ))
                    .push(
                        widget::button::link(fl!("edit"))
                            .font_size(12)
                            .on_press(AppAccountsMessage::EditAccount(item.to_owned())),
                    )
                    .push(
                        widget::button::link(fl!("remove"))
                            .font_size(12)
                            .on_press(AppAccountsMessage::DeleteAccount(item.to_owned())),
                    )
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

            widget::container(
                widget::column::with_children(vec![widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .push(widget::text::title3(fl!(
                        "accounts-with-count",
                        count = self.accounts.len()
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
                .push(accounts_widget),
            )
            .into()
        }
    }
    pub fn update(&mut self, message: AppAccountsMessage) -> Task<Message> {
        let mut commands = Vec::new();
        match message {
            AppAccountsMessage::AddAccount => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::AddAccount)
                }));
            }
            AppAccountsMessage::EditAccount(account) => {
                self.account_placeholder = Some(account.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::EditAccount(account.clone()))
                }));
            }
            AppAccountsMessage::DeleteAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::OpenRemoveAccountDialog(account.clone()))
                }));
            }
            AppAccountsMessage::RefreshBookmarksForAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::StartRefreshBookmarksForAccount(
                        account.clone(),
                    ))
                }));
            }
            AppAccountsMessage::OpenExternalURL(url) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::OpenExternalUrl(url.clone()))
                }));
            }
        }
        Task::batch(commands)
    }
}

pub fn add_account<'a>(account: Account) -> Element<'a, Message> {
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
    let tls_widget_checkbox =
        widget::checkbox(fl!("tls"), account.tls).on_toggle(Message::SetAccountTLS);
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::CompleteAddAccount(account)),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    widget::column()
        .spacing(space_xxs)
        .push(display_name_widget_title)
        .push(display_name_widget_text_input)
        .push(instance_widget_title)
        .push(instance_widget_text_input)
        .push(api_key_widget_title)
        .push(api_key_widget_text_input)
        .push(widget::Space::new(0, 10))
        .push(tls_widget_checkbox)
        .push(widget::Space::new(0, 10))
        .push(buttons_widget_container)
        .into()
}

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
    let tls_widget_checkbox =
        widget::checkbox(fl!("tls"), account.tls).on_toggle(Message::SetAccountTLS);
    let enable_shared_widget_text = if account.enable_sharing {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("enabled-sharing")))
                .push(widget::icon::from_name("dialog-information-symbolic").size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    } else {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("disabled-sharing")))
                .push(widget::icon::from_name("dialog-information-symbolic").size(18)),
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
                .push(widget::icon::from_name("dialog-information-symbolic").size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    } else {
        widget::tooltip(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::text::body(fl!("disabled-public-sharing")))
                .push(widget::icon::from_name("dialog-information-symbolic").size(18)),
            widget::container(widget::text::body(fl!("setting-managed-externally"))),
            tooltip::Position::FollowCursor,
        )
        .padding(10)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::UpdateAccount(account)),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    widget::column()
        .spacing(space_xxs)
        .push(display_name_widget_title)
        .push(display_name_widget_text_input)
        .push(instance_widget_title)
        .push(instance_widget_text_input)
        .push(api_key_widget_title)
        .push(api_key_widget_text_input)
        .push(widget::Space::new(0, 10))
        .push(tls_widget_checkbox)
        .push(widget::Space::new(0, 10))
        .push(enable_shared_widget_text)
        .push(enable_public_shared_widget_text)
        .push(widget::Space::new(0, 10))
        .push(buttons_widget_container)
        .into()
}
