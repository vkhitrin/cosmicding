use crate::app::Message;
use crate::fl;
use crate::models::account::Account;
use cosmic::iced::Length;
use cosmic::{
    cosmic_theme,
    iced::{self, Alignment},
    theme,
    widget::{self},
    Apply, Command, Element,
};
use iced::alignment::{Horizontal, Vertical};

#[derive(Debug, Clone)]
pub enum AccountsMessage {
    AddAccount,
    EditAccount(Account),
    DeleteAccount(Account),
    RefreshBookmarksForAccount(Account),
    OpenExternalURL(String),
}

#[derive(Debug, Clone)]
pub struct AccountsView {
    pub accounts: Vec<Account>,
    account_placeholder: Option<Account>,
}

impl Default for AccountsView {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            account_placeholder: None,
        }
    }
}

impl AccountsView {
    pub fn view<'a>(&'a self) -> Element<'a, AccountsMessage> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts.is_empty() {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("contact-new-symbolic")
                        .size(64)
                        .into(),
                    widget::text::title3(fl!("no-accounts")).into(),
                    widget::button::text(fl!("add-account"))
                        .style(widget::button::Style::Standard)
                        .on_press(AccountsMessage::AddAccount)
                        .into(),
                ])
                .spacing(20)
                .align_items(Alignment::Center),
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

            // TODO: (vkhitrin) Implement visual indicator for last sync status.
            for item in self.accounts.iter() {
                // Mandatory first row - details
                let mut columns = Vec::new();
                columns.push(
                    widget::row::with_capacity(1)
                        .spacing(spacing.space_xxs)
                        .push(widget::text(item.display_name.clone()))
                        .push(widget::horizontal_space(Length::Fill))
                        .push(
                            widget::button::link(item.instance.clone())
                                .on_press(AccountsMessage::OpenExternalURL(item.instance.clone())),
                        )
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                        ])
                        .align_items(Alignment::Center)
                        .into(),
                );
                // Mandatory third row - actions
                let actions_row = widget::row::with_capacity(1)
                    .spacing(spacing.space_xxs)
                    .push(
                        widget::button::link(fl!("refresh"))
                            .on_press(AccountsMessage::RefreshBookmarksForAccount(item.to_owned())),
                    )
                    .push(
                        widget::button::link(fl!("edit"))
                            .on_press(AccountsMessage::EditAccount(item.to_owned())),
                    )
                    .push(
                        widget::button::link(fl!("remove"))
                            .on_press(AccountsMessage::DeleteAccount(item.to_owned())),
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
                    .align_items(Alignment::Center)
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
                    .push(widget::horizontal_space(Length::Fill))
                    .push(
                        widget::button::text(fl!("add-account"))
                            .on_press(AccountsMessage::AddAccount)
                            .style(theme::Button::Standard),
                    )
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into()])
                .push(accounts_widget),
            )
            .into()
        }
    }
    pub fn update(&mut self, message: AccountsMessage) -> Command<Message> {
        let mut commands = Vec::new();
        match message {
            AccountsMessage::AddAccount => {
                commands.push(Command::perform(async {}, |_| Message::AddAccount));
            }
            AccountsMessage::EditAccount(account) => {
                self.account_placeholder = Some(account.clone());
                commands.push(Command::perform(async {}, move |_| {
                    Message::EditAccount(account)
                }));
            }
            AccountsMessage::DeleteAccount(account) => {
                commands.push(Command::perform(async {}, move |_| {
                    Message::OpenRemoveAccountDialog(account)
                }));
            }
            AccountsMessage::RefreshBookmarksForAccount(account) => {
                commands.push(Command::perform(async {}, move |_| {
                    Message::StartRefreshBookmarksForAccount(account)
                }));
            }
            AccountsMessage::OpenExternalURL(url) => {
                commands.push(Command::perform(async {}, |_| {
                    Message::OpenExternalUrl(url)
                }));
            }
        }
        Command::batch(commands)
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
    let api_key_widget_text_input =
        widget::secure_input("Token", account.api_token.clone(), None, true)
            .on_input(Message::SetAccountAPIKey);
    let tls_widget_checkbox = widget::checkbox(fl!("tls"), account.tls, Message::SetAccountTLS);
    let buttons_widget_container = widget::container(
        widget::button::text(fl!("save"))
            .style(widget::button::Style::Standard)
            .on_press(Message::CompleteAddAccount(account)),
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
        .push(widget::vertical_space(Length::from(10)))
        .push(tls_widget_checkbox)
        .push(widget::vertical_space(Length::from(10)))
        .push(buttons_widget_container)
        .into()
}

pub fn edit_account<'a>(account: Account) -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let display_name_widget_title = widget::text::body(fl!("display-name"));
    let display_name_widget_text_input = widget::text_input("Name", account.display_name.clone())
        .on_input(Message::SetAccountDisplayName);
    let instance_widget_title = widget::text::body(fl!("instance"));
    let instance_widget_text_input = widget::text_input("Instance", account.instance.clone())
        .on_input(Message::SetAccountInstance);
    let api_key_widget_title = widget::text::body(fl!("api-key"));
    let api_key_widget_text_input =
        widget::secure_input("Token", account.api_token.clone(), None, true)
            .on_input(Message::SetAccountAPIKey);
    let tls_widget_checkbox = widget::checkbox(fl!("tls"), account.tls, Message::SetAccountTLS);
    let buttons_widget_container = widget::container(
        widget::button::text(fl!("save"))
            .style(widget::button::Style::Standard)
            .on_press(Message::UpdateAccount(account)),
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
        .push(widget::vertical_space(Length::from(10)))
        .push(tls_widget_checkbox)
        .push(widget::vertical_space(Length::from(10)))
        .push(buttons_widget_container)
        .into()
}
