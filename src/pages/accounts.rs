use crate::app::{
    actions::{AccountsAction, ApplicationAction},
    ApplicationState, REFRESH_ICON,
};
use crate::fl;
use crate::{
    models::{account::Account, db_cursor::AccountsPaginationCursor, sync_status::SyncStatus},
    style::button::ButtonStyle,
};
use chrono::{DateTime, Local};
use cosmic::{
    app::Task,
    cosmic_theme,
    iced::{Alignment, Length},
    iced_widget::tooltip,
    style, theme,
    widget::{self},
    Apply, Element,
};
use cosmic_time::{anim, Timeline};

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
        sync_status: SyncStatus,
        accounts_cursor: &AccountsPaginationCursor,
        refresh_animation: &Timeline,
    ) -> Element<'_, AccountsAction> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts.is_empty() {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("contact-new-symbolic")
                        .size(64)
                        .into(),
                    widget::text::title3(fl!("no-accounts")).into(),
                    widget::button::standard(fl!("add-account"))
                        .on_press(AccountsAction::AddAccount)
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
            let mut accounts = widget::list::list_column()
                .style(style::Container::Background)
                .list_item_padding(spacing.space_none)
                .divider_padding(spacing.space_none)
                .spacing(spacing.space_xxxs)
                .padding(spacing.space_none);

            for account in &self.accounts {
                let local_time: DateTime<Local> = DateTime::from(
                    DateTime::from_timestamp(account.last_sync_timestamp, 0).expect(""),
                );

                let refresh_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("refresh"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => {
                        if account.enabled {
                            widget::button::link(fl!("refresh")).font_size(12).on_press(
                                AccountsAction::RefreshBookmarksForAccount(account.to_owned()),
                            )
                        } else {
                            widget::button::link(fl!("refresh"))
                                .font_size(12)
                                .class(ButtonStyle::DisabledLink(false).into())
                        }
                    }
                };
                let edit_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("edit"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("edit"))
                        .font_size(12)
                        .on_press(AccountsAction::EditAccount(account.to_owned())),
                };
                let remove_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .on_press(AccountsAction::DeleteAccount(account.to_owned())),
                };
                // Mandatory first row - details
                let mut columns = Vec::new();
                columns.push(
                    widget::row::with_capacity(2)
                        .push(widget::icon(widget::icon::from_svg_bytes(include_bytes!(
                            "../../res/icons/linkding-logo.svg"
                        ))))
                        .push(
                            widget::button::link(format!(
                                "{} ({})",
                                account.display_name.clone(),
                                if account.enabled {
                                    fl!("enabled")
                                } else {
                                    fl!("disabled")
                                }
                            ))
                            .spacing(spacing.space_xxxs)
                            .trailing_icon(true)
                            .icon_size(11)
                            .tooltip(account.instance.clone())
                            .on_press(AccountsAction::OpenExternalURL(account.instance.clone())),
                        )
                        .spacing(spacing.space_xxs)
                        .padding([
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_none,
                            spacing.space_xxxs,
                        ])
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory second row - details
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(widget::container(widget::column::with_children(vec![
                            if account.enable_sharing {
                                widget::text::body(fl!("enabled-sharing")).into()
                            } else {
                                widget::text::body(fl!("disabled-sharing")).into()
                            },
                            if account.enable_public_sharing {
                                widget::text::body(fl!("enabled-public-sharing")).into()
                            } else {
                                widget::text::body(fl!("disabled-public-sharing")).into()
                            },
                        ])))
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Mandatory third row - actions
                let actions_row = widget::row::with_capacity(3)
                    .spacing(spacing.space_xs)
                    .push(refresh_button)
                    .push(edit_button)
                    .push(remove_button);
                columns.push(
                    actions_row
                        .padding([
                            spacing.space_none,
                            spacing.space_xxs,
                            spacing.space_none,
                            spacing.space_none,
                        ])
                        .into(),
                );
                // Mandatory fourth row - sync timestamp + status
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                            spacing.space_xxxs,
                        ])
                        .push(
                            widget::text::body(format!(
                                "{}: {} ({})",
                                fl!("last-sync-time"),
                                local_time.format("%a, %d %b %Y %H:%M:%S"),
                                if account.last_sync_status {
                                    fl!("successful")
                                } else {
                                    fl!("failed")
                                }
                            ))
                            .size(12),
                        )
                        .align_y(Alignment::Center)
                        .into(),
                );
                let account_container = widget::container(widget::column::with_children(columns))
                    .class(theme::Container::Background);

                accounts = accounts.add(account_container);
            }

            let accounts_widget = widget::column::with_capacity(1)
                .spacing(spacing.space_xxs)
                .push(accounts)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill);

            let navigation_previous_button = widget::button::standard(fl!("previous"))
                .on_press_maybe(if accounts_cursor.current_page > 1 {
                    Some(AccountsAction::DecrementPageIndex)
                } else {
                    None
                })
                .leading_icon(widget::icon::from_name("go-previous-symbolic"));
            let navigation_next_button = widget::button::standard(fl!("next"))
                .on_press_maybe(
                    if accounts_cursor.current_page < accounts_cursor.total_pages {
                        Some(AccountsAction::IncrementPageIndex)
                    } else {
                        None
                    },
                )
                .trailing_icon(widget::icon::from_name("go-next-symbolic"));
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
                        spacing.space_xxs,
                        spacing.space_none,
                    ])
                    .push(navigation_next_button)
                    .push(widget::horizontal_space())
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into(),
            ]));
            let animation_widget = match app_state {
                ApplicationState::Refreshing => anim![REFRESH_ICON, &refresh_animation, 16],
                _ => match sync_status {
                    SyncStatus::InProgress => anim![REFRESH_ICON, &refresh_animation, 16],
                    SyncStatus::Warning => cosmic::widget::icon(
                        widget::icon::from_name("dialog-warning-symbolic").handle(),
                    )
                    .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(
                        move |theme: &cosmic::theme::Theme| {
                            let cosmic = theme.cosmic();
                            cosmic::iced_widget::svg::Style {
                                color: Some(cosmic.warning.base.into()),
                            }
                        },
                    )))
                    .size(16),
                    SyncStatus::Failed => widget::icon::from_name("dialog-error-symbolic")
                        .size(16)
                        .into(),
                    SyncStatus::Successful => cosmic::widget::icon(
                        widget::icon::from_name("checkbox-checked-symbolic").handle(),
                    )
                    .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(
                        move |theme: &cosmic::theme::Theme| {
                            let cosmic = theme.cosmic();
                            cosmic::iced_widget::svg::Style {
                                color: Some(cosmic.success.base.into()),
                            }
                        },
                    )))
                    .size(16),
                    SyncStatus::None => cosmic::widget::icon(
                        widget::icon::from_name("media-record-symbolic").handle(),
                    )
                    .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(
                        move |theme: &cosmic::theme::Theme| {
                            let cosmic = theme.cosmic();
                            cosmic::iced_widget::svg::Style {
                                color: Some(cosmic.background.base.into()),
                            }
                        },
                    )))
                    .size(16),
                },
            };

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
                        spacing.space_xxs,
                        spacing.space_none,
                    ])
                    .push(animation_widget)
                    .push(widget::horizontal_space())
                    .push(
                        widget::button::standard(fl!("add-account"))
                            .on_press(AccountsAction::AddAccount),
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
    pub fn update(&mut self, message: AccountsAction) -> Task<ApplicationAction> {
        let mut commands = Vec::new();
        match message {
            AccountsAction::AddAccount => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::AddAccountForm)
                }));
            }
            AccountsAction::EditAccount(account) => {
                self.account_placeholder = Some(account.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::EditAccountForm(account.clone()))
                }));
            }
            AccountsAction::DeleteAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::OpenRemoveAccountDialog(account.clone()))
                }));
            }
            AccountsAction::RefreshBookmarksForAccount(account) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::StartRefreshBookmarksForAccount(
                        account.clone(),
                    ))
                }));
            }
            AccountsAction::OpenExternalURL(url) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::OpenExternalUrl(url.clone()))
                }));
            }
            AccountsAction::IncrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::IncrementPageIndex(
                        "accounts".to_string(),
                    ))
                }));
            }
            AccountsAction::DecrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::DecrementPageIndex(
                        "accounts".to_string(),
                    ))
                }));
            }
        }
        Task::batch(commands)
    }
}

pub fn add_account<'a>(account: Account) -> Element<'a, ApplicationAction> {
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let display_name_widget_title = widget::text::body(fl!("display-name"));
    let display_name_widget_text_input = widget::text_input("Name", account.display_name.clone())
        .on_input(ApplicationAction::SetAccountDisplayName);
    let instance_widget_title = widget::text::body(fl!("instance"));
    let instance_widget_text_input = widget::text_input("Instance", account.instance.clone())
        .on_input(ApplicationAction::SetAccountInstance);
    let api_key_widget_title = widget::text::body(fl!("api-key"));
    let api_key_widget_text_input = widget::text_input(fl!("token"), account.api_token.clone())
        .on_input(ApplicationAction::SetAccountAPIKey)
        .password();
    let trust_invalid_certs_widget_toggler = widget::toggler(account.trust_invalid_certs)
        .on_toggle(ApplicationAction::SetAccountTrustInvalidCertificates)
        .spacing(10)
        .label(fl!("trust-invalid-certificates"));
    let account_status_toggler = widget::toggler(account.enabled)
        .on_toggle(ApplicationAction::SetAccountStatus)
        .spacing(10)
        .label(fl!("enabled"));
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(ApplicationAction::StartAddAccount(account)),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("user-available-symbolic"))
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
                .push(widget::icon::from_name("network-server-symbolic"))
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
                .push(widget::icon::from_name("system-lock-screen-symbolic"))
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
                .push(trust_invalid_certs_widget_toggler)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_none,
                    spacing.space_none,
                ]),
        )
        .push(
            widget::row::with_capacity(1)
                .push(account_status_toggler)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_none,
                    spacing.space_xs,
                    spacing.space_none,
                ]),
        )
        .push(buttons_widget_container)
        .into()
}

#[allow(clippy::too_many_lines)]
pub fn edit_account<'a>(account: Account) -> Element<'a, ApplicationAction> {
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let display_name_widget_title = widget::text::body(fl!("display-name"));
    let display_name_widget_text_input = widget::text_input("Name", account.display_name.clone())
        .on_input(ApplicationAction::SetAccountDisplayName);
    let instance_widget_title = widget::text::body(fl!("instance"));
    let instance_widget_text_input = widget::text_input("Instance", account.instance.clone())
        .on_input(ApplicationAction::SetAccountInstance);
    let api_key_widget_title = widget::text::body(fl!("api-key"));
    let api_key_widget_text_input = widget::text_input(fl!("token"), account.api_token.clone())
        .on_input(ApplicationAction::SetAccountAPIKey)
        .password();
    let trust_invalid_certs_widget_toggler = widget::toggler(account.trust_invalid_certs)
        .on_toggle(ApplicationAction::SetAccountTrustInvalidCertificates)
        .spacing(10)
        .label(fl!("trust-invalid-certificates"));
    let account_status_toggler = widget::toggler(account.enabled)
        .on_toggle(ApplicationAction::SetAccountStatus)
        .spacing(10)
        .label(fl!("enabled"));
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
        widget::button::standard(fl!("save"))
            .on_press(ApplicationAction::StartEditAccount(account)),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("user-available-symbolic"))
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
                .push(widget::icon::from_name("network-server-symbolic"))
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
                .push(widget::icon::from_name("system-lock-screen-symbolic"))
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
                .push(trust_invalid_certs_widget_toggler)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_none,
                    spacing.space_none,
                ]),
        )
        .push(
            widget::row::with_capacity(1)
                .push(account_status_toggler)
                .padding([
                    spacing.space_xxxs,
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
