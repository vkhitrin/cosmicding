use crate::app::{
    actions::{AccountsAction, ApplicationAction},
    ApplicationState, REFRESH_ICON,
};
use crate::fl;
use crate::{
    models::{
        account::Account, db_cursor::AccountsPaginationCursor, operation::OperationProgress,
        provider::Provider, sync_status::SyncStatus,
    },
    provider::ALLOWED_PROVIDERS,
    style::button::ButtonStyle,
    widgets::progress_info::{operation_progress_widget, ProgressInfo},
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
        operation_progress: Option<&OperationProgress>,
    ) -> Element<'_, AccountsAction> {
        let spacing = theme::active().cosmic().spacing;
        let add_button = match app_state {
            ApplicationState::Ready | ApplicationState::NoEnabledRemoteAccounts => {
                widget::button::standard(fl!("add-account")).on_press(AccountsAction::AddAccount)
            }
            _ => widget::button::standard(fl!("add-account")),
        };
        let mut accounts = widget::list::list_column()
            .style(style::Container::Background)
            .list_item_padding(spacing.space_none)
            .divider_padding(spacing.space_none)
            .spacing(spacing.space_xxxs)
            .padding(spacing.space_none);

        for account in &self.accounts {
            let local_time: DateTime<Local> =
                DateTime::from(DateTime::from_timestamp(account.last_sync_timestamp, 0).expect(""));

            let mut columns = Vec::new();

            // First row - account title with provider icon and badges
            let badge_radius = spacing.space_xxxs;
            let mut all_badges: Vec<Element<'_, AccountsAction>> = Vec::new();

            let status_background_color = if account.enabled {
                cosmic::iced::Color::from_rgba(0.0, 0.6, 0.0, 0.2)
            } else {
                cosmic::iced::Color::from_rgba(0.6, 0.0, 0.0, 0.2)
            };

            let status_badge = widget::container(
                widget::text::caption(if account.enabled {
                    fl!("enabled")
                } else {
                    fl!("disabled")
                })
                .size(10),
            )
            .padding([spacing.space_xxxs, spacing.space_xxs])
            .style(move |_theme| widget::container::Style {
                background: Some(cosmic::iced::Background::Color(status_background_color)),
                border: cosmic::iced::Border {
                    radius: badge_radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
            all_badges.push(status_badge.into());

            // Sharing badges (only for linkding)
            if !account.is_local_provider() {
                if account.enable_sharing {
                    let sharing_badge =
                        widget::container(widget::text::caption(fl!("sharing")).size(10))
                            .padding([spacing.space_xxxs, spacing.space_xxs])
                            .style(move |_theme| widget::container::Style {
                                background: Some(cosmic::iced::Background::Color(
                                    cosmic::iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                                )),
                                border: cosmic::iced::Border {
                                    radius: badge_radius.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                    all_badges.push(sharing_badge.into());
                }

                if account.enable_public_sharing {
                    let public_sharing_badge =
                        widget::container(widget::text::caption(fl!("public-sharing")).size(10))
                            .padding([spacing.space_xxxs, spacing.space_xxs])
                            .style(move |_theme| widget::container::Style {
                                background: Some(cosmic::iced::Background::Color(
                                    cosmic::iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                                )),
                                border: cosmic::iced::Border {
                                    radius: badge_radius.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                    all_badges.push(public_sharing_badge.into());
                }
            }

            let provider_type_badge_content = widget::container(
                widget::text::caption(if account.is_local_provider() {
                    "Local"
                } else {
                    "Remote"
                })
                .size(10),
            )
            .padding([spacing.space_xxxs, spacing.space_xxs])
            .style(move |_theme| widget::container::Style {
                background: Some(cosmic::iced::Background::Color(
                    cosmic::iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                )),
                border: cosmic::iced::Border {
                    radius: badge_radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            let tooltip_text = if let Some(version) = &account.provider_version {
                format!("{}: {}", account.provider(), version)
            } else {
                account.provider().to_string()
            };

            let provider_type_badge = widget::tooltip(
                provider_type_badge_content,
                widget::container(widget::text::body(tooltip_text)),
                tooltip::Position::Top,
            )
            .padding(5);

            all_badges.push(provider_type_badge.into());

            let provider_icon = widget::icon(account.provider().svg_icon());
            let account_name_widget: Element<'_, AccountsAction> = if account.is_local_provider() {
                widget::text::body(account.display_name.clone()).into()
            } else {
                widget::button::link(account.display_name.clone())
                    .spacing(spacing.space_xxxs)
                    .trailing_icon(true)
                    .icon_size(11)
                    .tooltip(account.instance.clone())
                    .on_press(AccountsAction::OpenExternalURL(account.instance.clone()))
                    .into()
            };

            columns.push(
                widget::row::with_capacity(4)
                    .push(provider_icon)
                    .push(account_name_widget)
                    .push(widget::horizontal_space())
                    .push(widget::row::with_children(all_badges).spacing(spacing.space_xxs))
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

            // Second row - actions (only for non-local providers)
            if !account.is_local_provider() {
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
                let toggle_status_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(if account.enabled {
                        fl!("disable")
                    } else {
                        fl!("enable")
                    })
                    .font_size(12)
                    .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(if account.enabled {
                        fl!("disable")
                    } else {
                        fl!("enable")
                    })
                    .font_size(12)
                    .on_press(AccountsAction::ToggleAccountStatus(account.to_owned())),
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
                let actions_row = widget::row::with_capacity(4)
                    .spacing(spacing.space_xs)
                    .push(refresh_button)
                    .push(toggle_status_button)
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
            }
            // Third row - sync timestamp + status (only for remote providers)
            if !account.is_local_provider() {
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
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
            }
            let account_container = widget::container(widget::column::with_children(columns))
                .class(theme::Container::Background)
                .padding(if account.is_local_provider() {
                    [
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_xxs,
                        spacing.space_none,
                    ]
                } else {
                    [
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_none,
                    ]
                });

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
            ApplicationState::NoEnabledRemoteAccounts => cosmic::widget::icon(
                widget::icon::from_name("network-wireless-disconnected-symbolic").handle(),
            )
            .size(16),
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
                SyncStatus::None => {
                    cosmic::widget::icon(widget::icon::from_name("media-record-symbolic").handle())
                        .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(
                            move |theme: &cosmic::theme::Theme| {
                                let cosmic = theme.cosmic();
                                cosmic::iced_widget::svg::Style {
                                    color: Some(cosmic.background.base.into()),
                                }
                            },
                        )))
                        .size(16)
                }
            },
        };

        let mut main_column = widget::column::with_children(vec![widget::row::with_capacity(2)
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
            .push(add_button)
            .width(Length::Fill)
            .apply(widget::container)
            .into()]);

        // Add operation progress widget if any operation is active
        if let Some(progress) = operation_progress {
            let progress_info = ProgressInfo {
                total: progress.total,
                current: progress.current,
                label: progress.operation_label.clone(),
                cancellable: progress.cancellable,
            };

            let progress_widget = operation_progress_widget(
                &progress_info,
                if progress.cancellable {
                    Some(AccountsAction::CancelImport(progress.operation_id))
                } else {
                    None
                },
            );

            main_column = main_column.push(progress_widget);
        }

        main_column = main_column
            .push(accounts_widget)
            .push(page_navigation_widget);

        widget::container(main_column).into()
    }
    pub fn update(&mut self, message: AccountsAction) -> Task<ApplicationAction> {
        let mut commands = Vec::new();
        match message {
            AccountsAction::AddAccount => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::AddAccountForm)
                }));
            }
            AccountsAction::CancelImport(import_id) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::CancelImportBookmarks(import_id))
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
            AccountsAction::ToggleAccountStatus(account) => {
                let mut updated_account = account.clone();
                updated_account.enabled = !updated_account.enabled;
                commands.push(Task::perform(async {}, move |()| {
                    let mut acc = account.clone();
                    acc.enabled = !acc.enabled;
                    cosmic::Action::App(ApplicationAction::StartEditAccount(acc))
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
    let provider_widget_title = widget::text::body("Provider");
    let provider_selected = match account.provider() {
        Provider::Linkding => Some(0),
        Provider::Cosmicding => None,
    };
    let provider_dropdown = widget::dropdown(ALLOWED_PROVIDERS, provider_selected, |idx| {
        let provider_name = ALLOWED_PROVIDERS.get(idx).unwrap_or(&"Linkding");
        ApplicationAction::SetAccountProvider(Provider::from_str(provider_name))
    });
    let provider_icon = widget::icon(account.provider().svg_icon()).size(16);
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
                .push(widget::icon::from_name("preferences-system-symbolic"))
                .push(provider_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(provider_icon)
                .push(provider_dropdown)
                .align_y(Alignment::Center),
        )
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
    let provider_widget_title = widget::text::body("Provider");
    let provider_text = account.provider().to_string();
    let provider_widget_text_input = widget::text_input("", provider_text);
    let provider_icon = widget::icon(account.provider().svg_icon()).size(16);
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
                .push(provider_icon.clone())
                .push(provider_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(provider_widget_text_input)
                .align_y(Alignment::Center),
        )
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
        .push(enable_shared_widget_text)
        .push(enable_public_shared_widget_text)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}
