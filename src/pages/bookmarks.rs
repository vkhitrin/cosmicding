use crate::{
    app::{
        actions::{ApplicationAction, BookmarksAction},
        ApplicationState, REFRESH_ICON,
    },
    fl,
    models::{
        account::Account, bookmarks::Bookmark, db_cursor::BookmarksPaginationCursor,
        sync_status::SyncStatus,
    },
    style::{button::ButtonStyle, text_editor::text_editor_class},
};
use chrono::{DateTime, Local};
use cosmic::{
    app::Task,
    cosmic_theme,
    iced::{Alignment, Length},
    iced_core::text,
    style, theme,
    widget::{self},
    Apply, Element,
};
use cosmic_time::{anim, Timeline};

#[derive(Debug, Default, Clone)]
pub struct PageBookmarksView {
    bookmark_placeholder: Option<Bookmark>,
    pub bookmarks: Vec<Bookmark>,
    pub search_id: Option<widget::Id>,
    query_placeholder: String,
}

impl PageBookmarksView {
    #[allow(clippy::too_many_lines)]
    pub fn view(
        &self,
        app_state: ApplicationState,
        sync_status: SyncStatus,
        bookmarks_cursor: &BookmarksPaginationCursor,
        no_accounts: bool,
        refresh_animation: &Timeline,
    ) -> Element<'_, BookmarksAction> {
        let spacing = theme::active().cosmic().spacing;
        if no_accounts {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("web-browser-symbolic")
                        .size(64)
                        .into(),
                    widget::text::title3(fl!("no-accounts")).into(),
                    widget::button::standard(fl!("open-accounts-page"))
                        .on_press(BookmarksAction::OpenAccountsPage)
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
            let mut bookmarks = widget::list::list_column()
                .style(style::Container::Background)
                .list_item_padding(spacing.space_none)
                .divider_padding(spacing.space_none)
                .spacing(spacing.space_xxxs)
                .padding(spacing.space_none);

            for bookmark in &self.bookmarks {
                let bookmark_account_id = bookmark.user_account_id.unwrap();
                let date_added: DateTime<Local> =
                    bookmark.date_added.clone().unwrap().parse().expect("");
                let mut columns = Vec::new();
                let favicon_data = bookmark
                    .favicon_cached
                    .as_ref()
                    .filter(|cached| !cached.favicon_data.is_empty())
                    .map(|cached| cached.favicon_data.clone());
                let favicon: widget::image::Handle = if let Some(data) = favicon_data {
                    widget::image::Handle::from_bytes(data)
                } else {
                    let placeholder: &[u8] =
                        include_bytes!("../../res/icons/favicon_placeholder.png");
                    widget::image::Handle::from_bytes(placeholder)
                };
                // Mandatory first row - title
                columns.push(
                    widget::row::with_capacity(2)
                        .spacing(spacing.space_xxs)
                        .padding([
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_none,
                            spacing.space_xxxs,
                        ])
                        .push(widget::image(favicon).width(16))
                        .push(
                            widget::button::link(bookmark.title.clone())
                                .spacing(spacing.space_xxxs)
                                .trailing_icon(true)
                                .icon_size(11)
                                .tooltip(bookmark.url.clone())
                                .on_press(BookmarksAction::OpenExternalURL(bookmark.url.clone())),
                        )
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Optional second row - description
                if !bookmark.description.is_empty() {
                    columns.push(
                        widget::row::with_capacity(2)
                            .spacing(spacing.space_xs)
                            .padding([
                                spacing.space_xxxs,
                                spacing.space_xxs,
                                if bookmark.tag_names.is_empty() {
                                    spacing.space_xxxs
                                } else {
                                    spacing.space_none
                                },
                                spacing.space_xxxs,
                            ])
                            .push(widget::text(bookmark.description.clone()))
                            .align_y(Alignment::Start)
                            .into(),
                    );
                }
                // Optional third row - tags
                if !bookmark.tag_names.is_empty() {
                    columns.push(
                        widget::row::with_capacity(2)
                            .spacing(spacing.space_xs)
                            .padding([
                                if bookmark.description.is_empty() {
                                    spacing.space_xxxs
                                } else {
                                    spacing.space_xxs
                                },
                                spacing.space_xxs,
                                spacing.space_xxxs,
                                spacing.space_xxxs,
                            ])
                            .push(
                                widget::text::body(
                                    bookmark
                                        .tag_names
                                        .iter()
                                        .map(|s| format!("#{s}"))
                                        .collect::<Vec<String>>()
                                        .join(" "),
                                )
                                .size(12),
                            )
                            .align_y(Alignment::Center)
                            .into(),
                    );
                }
                // Mandatory fourth row - actions
                let edit_bookmark_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("edit"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("edit")).font_size(12).on_press(
                        BookmarksAction::EditBookmark(bookmark_account_id, bookmark.to_owned()),
                    ),
                };
                let remove_bookmark_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("remove")).font_size(12).on_press(
                        BookmarksAction::DeleteBookmark(bookmark_account_id, bookmark.to_owned()),
                    ),
                };
                let notes_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("notes"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("notes"))
                        .font_size(12)
                        .on_press(BookmarksAction::ViewNotes(bookmark.clone())),
                };
                let snapshot_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("snapshot"))
                        .spacing(spacing.space_xxxs)
                        .trailing_icon(true)
                        .font_size(12)
                        .icon_size(11)
                        .tooltip(bookmark.web_archive_snapshot_url.clone())
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("snapshot"))
                        .spacing(spacing.space_xxxs)
                        .trailing_icon(true)
                        .font_size(12)
                        .icon_size(11)
                        .tooltip(bookmark.web_archive_snapshot_url.clone())
                        .on_press(BookmarksAction::OpenExternalURL(
                            bookmark.web_archive_snapshot_url.clone(),
                        )),
                };
                let mut actions_row = widget::row::with_capacity(1).spacing(spacing.space_xxs);
                if bookmark.is_owner == Some(true) {
                    actions_row = actions_row.push(edit_bookmark_button);
                    actions_row = actions_row.push(remove_bookmark_button);
                }
                if !bookmark.notes.is_empty() {
                    actions_row = actions_row.push(notes_button);
                }
                if !bookmark.web_archive_snapshot_url.is_empty() {
                    actions_row = actions_row.push(snapshot_button);
                }
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
                // Mandatory fifth row - details
                let mut details_row = widget::row::with_capacity(1).spacing(spacing.space_xxs);
                details_row = details_row.push(
                    widget::text(date_added.format("%a, %d %b %Y %H:%M:%S").to_string()).size(12),
                );
                if bookmark.is_archived {
                    details_row = details_row
                        .push(widget::icon::from_name("mail-archive-symbolic").size(12))
                        .push(widget::text(fl!("archived")).size(12));
                }
                if bookmark.unread {
                    details_row = details_row
                        .push(widget::icon::from_name("x-office-spreadsheet-symbolic").size(12))
                        .push(widget::text(fl!("unread")).size(12));
                }
                if bookmark.shared {
                    details_row = details_row
                        .push(widget::icon::from_name("emblem-shared-symbolic").size(12))
                        .push(if bookmark.is_owner == Some(true) {
                            widget::text(fl!("sharing")).size(12)
                        } else {
                            widget::text(fl!("shared")).size(12)
                        });
                }
                columns.push(
                    details_row
                        .push(widget::horizontal_space())
                        .align_y(Alignment::Center)
                        .spacing(spacing.space_xxs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_xxxs,
                        ])
                        .into(),
                );

                let bookmark_container = widget::container(widget::column::with_children(columns))
                    .padding(spacing.space_none)
                    .class(theme::Container::Background);

                bookmarks = bookmarks.add(bookmark_container);
            }

            let refresh_button = if !self.query_placeholder.is_empty()
                || self.bookmarks.is_empty()
                || matches!(app_state, ApplicationState::Refreshing)
            {
                widget::button::standard(fl!("refresh"))
            } else {
                widget::button::standard(fl!("refresh")).on_press(BookmarksAction::RefreshBookmarks)
            };
            let mut new_bookmark_button = widget::button::standard(fl!("add-bookmark"));

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

            let mut search_input_widget =
                widget::text_input::search_input(fl!("search"), self.query_placeholder.clone())
                    .id(self.search_id.clone().unwrap());

            if matches!(app_state, ApplicationState::Ready) {
                new_bookmark_button = new_bookmark_button.on_press(BookmarksAction::AddBookmark);
            }
            if !matches!(app_state, ApplicationState::NoEnabledAccounts) {
                search_input_widget = search_input_widget
                    .on_input(BookmarksAction::SearchBookmarks)
                    .on_clear(BookmarksAction::ClearSearch);
            }

            let bookmarks_widget = widget::column::with_capacity(1)
                .spacing(spacing.space_xxs)
                .push(bookmarks)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill);

            let navigation_previous_button = widget::button::standard(fl!("previous"))
                .on_press_maybe(if bookmarks_cursor.current_page > 1 {
                    Some(BookmarksAction::DecrementPageIndex)
                } else {
                    None
                })
                .leading_icon(widget::icon::from_name("go-previous-symbolic"));
            let navigation_next_button = widget::button::standard(fl!("next"))
                .on_press_maybe(
                    if bookmarks_cursor.current_page < bookmarks_cursor.total_pages {
                        Some(BookmarksAction::IncrementPageIndex)
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
                        bookmarks_cursor.current_page, bookmarks_cursor.total_pages
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

            widget::container(
                widget::column::with_children(vec![widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .push(widget::text::title3(fl!(
                        "bookmarks-with-count",
                        count = bookmarks_cursor.total_entries
                    )))
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_xxs,
                        spacing.space_none,
                    ])
                    .push(animation_widget)
                    .push(search_input_widget)
                    .push(refresh_button)
                    .push(new_bookmark_button)
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into()])
                .push(bookmarks_widget)
                .push(page_navigation_widget),
            )
            .into()
        }
    }
    pub fn update(&mut self, message: BookmarksAction) -> Task<ApplicationAction> {
        let mut commands = Vec::new();
        match message {
            BookmarksAction::OpenAccountsPage => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::OpenAccountsPage)
                }));
            }
            BookmarksAction::RefreshBookmarks => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::StartRefreshBookmarksForAllAccounts)
                }));
            }
            BookmarksAction::AddBookmark => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::AddBookmarkForm)
                }));
            }
            BookmarksAction::DeleteBookmark(account_id, bookmark) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::OpenRemoveBookmarkDialog(
                        account_id,
                        bookmark.clone(),
                    ))
                }));
            }
            BookmarksAction::EditBookmark(account_id, bookmark) => {
                self.bookmark_placeholder = Some(bookmark.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::EditBookmarkForm(
                        account_id,
                        bookmark.clone(),
                    ))
                }));
            }
            BookmarksAction::SearchBookmarks(query) => {
                self.query_placeholder.clone_from(&query);
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::SearchBookmarks(query.clone()))
                }));
            }
            BookmarksAction::ClearSearch => {
                if !self.query_placeholder.is_empty() {
                    self.query_placeholder = String::new();
                    commands.push(Task::perform(async {}, |()| {
                        cosmic::Action::App(ApplicationAction::SearchBookmarks(String::new()))
                    }));
                }
            }
            BookmarksAction::OpenExternalURL(url) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::OpenExternalUrl(url.clone()))
                }));
            }
            BookmarksAction::ViewNotes(bookmark) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::Action::App(ApplicationAction::ViewBookmarkNotes(bookmark.clone()))
                }));
            }
            BookmarksAction::EmptyMessage => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::Empty)
                }));
            }
            BookmarksAction::IncrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::IncrementPageIndex(
                        "bookmarks".to_string(),
                    ))
                }));
            }
            BookmarksAction::DecrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::Action::App(ApplicationAction::DecrementPageIndex(
                        "bookmarks".to_string(),
                    ))
                }));
            }
        }
        Task::batch(commands)
    }
}

#[allow(clippy::too_many_lines)]
pub fn new_bookmark<'a, 'b>(
    bookmark: Bookmark,
    bookmark_notes: &'a widget::text_editor::Content,
    bookmark_description: &'a widget::text_editor::Content,
    accounts: &'b [Account],
    selected_account_index: usize,
) -> Element<'a, ApplicationAction>
where
    'b: 'a,
{
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let account_widget_title = widget::text::body(fl!("account"));
    let account_widget_dropdown =
        widget::dropdown(accounts, Some(selected_account_index), move |idx| {
            ApplicationAction::AddBookmarkFormAccountIndex(idx)
        });
    let url_widget_title = widget::text::body(fl!("url"));
    let url_widget_text_input =
        widget::text_input("URL", bookmark.url.clone()).on_input(ApplicationAction::SetBookmarkURL);
    let title_widget_title = widget::text::body(fl!("title"));
    let title_widget_text_input = widget::text_input("Title", bookmark.title.clone())
        .on_input(ApplicationAction::SetBookmarkTitle);
    let description_widget_title = widget::text::body(fl!("description"));
    let description_widget_text_editor = widget::text_editor(bookmark_description)
        .height(120)
        .padding(spacing.space_xxs)
        .wrapping(text::Wrapping::WordOrGlyph)
        .on_action(ApplicationAction::InputBookmarkDescription)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )));
    let notes_widget_title = widget::text::body(fl!("notes"));
    let notes_widget_text_editor = widget::text_editor(bookmark_notes)
        .height(120)
        .padding(spacing.space_xxs)
        .wrapping(text::Wrapping::WordOrGlyph)
        .on_action(ApplicationAction::InputBookmarkNotes)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )));
    let tags_widget_title = widget::text::body(fl!("tags"));
    let tags_widget_subtext = widget::text::caption(fl!("tags-subtext"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(ApplicationAction::SetBookmarkTags);
    let archived_widget_toggler = widget::toggler(bookmark.is_archived)
        .on_toggle(ApplicationAction::SetBookmarkArchived)
        .spacing(10)
        .label(fl!("archived"));
    let unread_widget_toggler = widget::toggler(bookmark.unread)
        .on_toggle(ApplicationAction::SetBookmarkUnread)
        .spacing(10)
        .label(fl!("unread"));
    let shared_widget_toggler = widget::toggler(bookmark.shared)
        .spacing(10)
        .on_toggle_maybe(if accounts[selected_account_index].enable_sharing {
            Some(ApplicationAction::SetBookmarkShared)
        } else {
            None
        })
        .label(if accounts[selected_account_index].enable_sharing {
            fl!("shared")
        } else {
            fl!("shared-disabled")
        });
    let buttons_widget_container =
        widget::container(widget::button::standard(fl!("save")).on_press(
            ApplicationAction::StartAddBookmark(accounts[selected_account_index].clone(), bookmark),
        ))
        .width(Length::Fill)
        .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("user-available-symbolic"))
                .push(account_widget_title)
                .padding([
                    spacing.space_none,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(account_widget_dropdown)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("web-browser-symbolic"))
                .push(url_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(url_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("insert-text-symbolic"))
                .push(title_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(title_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("text-x-generic-symbolic"))
                .push(description_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(description_widget_text_editor)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("x-office-document-symbolic"))
                .push(notes_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(notes_widget_text_editor)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("mail-mark-important-symbolic"))
                .push(tags_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(tags_widget_subtext)
        .push(tags_widget_text_input)
        .push(widget::Space::new(0, 5))
        .push(archived_widget_toggler)
        .push(unread_widget_toggler)
        .push(shared_widget_toggler)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}

#[allow(clippy::too_many_lines)]
pub fn edit_bookmark<'a, 'b>(
    bookmark: Bookmark,
    bookmark_notes: &'a widget::text_editor::Content,
    bookmark_description: &'a widget::text_editor::Content,
    account: &'b Account,
) -> Element<'a, ApplicationAction>
where
    'b: 'a,
{
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let account_widget_title = widget::text::body(fl!("account"));
    let account_widget_text_input =
        widget::text_input(&account.display_name, &account.display_name);
    let url_widget_title = widget::text::body(fl!("url"));
    let url_widget_text_input =
        widget::text_input("URL", bookmark.url.clone()).on_input(ApplicationAction::SetBookmarkURL);
    let title_widget_title = widget::text::body(fl!("title"));
    let title_widget_text_input = widget::text_input("Title", bookmark.title.clone())
        .on_input(ApplicationAction::SetBookmarkTitle);
    let description_widget_title = widget::text::body(fl!("description"));
    let description_widget_text_editor = widget::text_editor(bookmark_description)
        .height(120)
        .padding(spacing.space_xxs)
        .wrapping(text::Wrapping::WordOrGlyph)
        .on_action(ApplicationAction::InputBookmarkDescription)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )));
    let notes_widget_title = widget::text::body(fl!("notes"));
    let notes_widget_text_editor = widget::text_editor(bookmark_notes)
        .height(120)
        .padding(spacing.space_xxs)
        .wrapping(text::Wrapping::WordOrGlyph)
        .on_action(ApplicationAction::InputBookmarkNotes)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )));
    let tags_widget_title = widget::text::body(fl!("tags"));
    let tags_widget_subtext = widget::text::caption(fl!("tags-subtext"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(ApplicationAction::SetBookmarkTags);
    let archived_widget_toggler = widget::toggler(bookmark.is_archived)
        .on_toggle(ApplicationAction::SetBookmarkArchived)
        .spacing(10)
        .label(fl!("archived"));
    let unread_widget_toggler = widget::toggler(bookmark.unread)
        .on_toggle(ApplicationAction::SetBookmarkUnread)
        .spacing(10)
        .label(fl!("unread"));
    let shared_widget_toggler = widget::toggler(bookmark.shared)
        .spacing(10)
        .on_toggle_maybe(if account.clone().enable_sharing {
            Some(ApplicationAction::SetBookmarkShared)
        } else {
            None
        })
        .label(if account.clone().enable_sharing {
            fl!("shared")
        } else {
            fl!("shared-disabled")
        });
    let buttons_widget_container =
        widget::container(widget::button::standard(fl!("save")).on_press(
            ApplicationAction::StartEditBookmark(account.clone(), bookmark),
        ))
        .width(Length::Fill)
        .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("user-available-symbolic"))
                .push(account_widget_title)
                .padding([
                    spacing.space_none,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(account_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("web-browser-symbolic"))
                .push(url_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(url_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("insert-text-symbolic"))
                .push(title_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(title_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("text-x-generic-symbolic"))
                .push(description_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(description_widget_text_editor)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("x-office-document-symbolic"))
                .push(notes_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(notes_widget_text_editor)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::from_name("mail-mark-important-symbolic"))
                .push(tags_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(tags_widget_subtext)
        .push(tags_widget_text_input)
        .push(widget::Space::new(0, 5))
        .push(archived_widget_toggler)
        .push(unread_widget_toggler)
        .push(shared_widget_toggler)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}

pub fn view_notes(bookmark_notes: &widget::text_editor::Content) -> Element<ApplicationAction> {
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let bookmark_notes_widget = widget::text_editor(bookmark_notes)
        .padding(spacing.space_xxs)
        .wrapping(text::Wrapping::WordOrGlyph)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )));

    widget::column()
        .spacing(space_xxs)
        .push(bookmark_notes_widget)
        .into()
}
