use crate::app::{icons::load_icon, ApplicationState, Message};
use crate::fl;
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;
use crate::models::db_cursor::BookmarksPaginationCursor;
use crate::style::button::ButtonStyle;
use chrono::{DateTime, Local};
use cosmic::iced::Length;
use cosmic::{
    app::command::Task,
    iced::Alignment,
    widget::{self},
    Apply, Element,
};
use cosmic::{cosmic_theme, theme};

#[derive(Debug, Default, Clone)]
pub struct PageBookmarksView {
    accounts_exist: bool,
    pub bookmarks: Vec<Bookmark>,
    bookmark_placeholder: Option<Bookmark>,
    query_placeholder: String,
}

#[derive(Debug, Clone)]
pub enum AppBookmarksMessage {
    AddBookmark,
    ClearSearch,
    DecrementPageIndex,
    DeleteBookmark(i64, Bookmark),
    EditBookmark(i64, Bookmark),
    EmptyMessage,
    IncrementPageIndex,
    OpenAccountsPage,
    OpenExternalURL(String),
    RefreshBookmarks,
    SearchBookmarks(String),
    ViewNotes(Bookmark),
}

impl PageBookmarksView {
    #[allow(clippy::too_many_lines)]
    pub fn view(
        &self,
        app_state: ApplicationState,
        bookmarks_cursor: &BookmarksPaginationCursor,
    ) -> Element<'_, AppBookmarksMessage> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts_exist {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::icon(load_icon("web-browser-symbolic"))
                        .size(64)
                        .into(),
                    widget::text::title3(fl!("no-accounts")).into(),
                    widget::button::standard(fl!("open-accounts-page"))
                        .on_press(AppBookmarksMessage::OpenAccountsPage)
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

            for item in &self.bookmarks {
                let bookmark_account_id = item.user_account_id.unwrap();
                let date_added: DateTime<Local> =
                    item.date_added.clone().unwrap().parse().expect("");
                let mut columns = Vec::new();
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
                        .push(widget::icon::icon(load_icon("web-browser-symbolic")))
                        .push(
                            widget::button::link(item.title.clone())
                                .spacing(spacing.space_xxxs)
                                .trailing_icon(true)
                                .icon_size(11)
                                .tooltip(item.url.clone())
                                .on_press(AppBookmarksMessage::OpenExternalURL(item.url.clone())),
                        )
                        .align_y(Alignment::Center)
                        .into(),
                );
                // Optional second row - description
                if !item.description.is_empty() {
                    columns.push(
                        widget::row::with_capacity(2)
                            .spacing(spacing.space_xs)
                            .padding([
                                spacing.space_xxxs,
                                spacing.space_xxs,
                                if item.tag_names.is_empty() {
                                    spacing.space_xxxs
                                } else {
                                    spacing.space_none
                                },
                                spacing.space_xxxs,
                            ])
                            .push(widget::icon::icon(load_icon("text-x-generic-symbolic")))
                            .push(widget::text(item.description.clone()))
                            .align_y(Alignment::Start)
                            .into(),
                    );
                }
                // Optional third row - tags
                if !item.tag_names.is_empty() {
                    columns.push(
                        widget::row::with_capacity(2)
                            .spacing(spacing.space_xs)
                            .padding([
                                if item.description.is_empty() {
                                    spacing.space_xxxs
                                } else {
                                    spacing.space_xxs
                                },
                                spacing.space_xxs,
                                spacing.space_xxxs,
                                spacing.space_xxxs,
                            ])
                            .push(widget::icon::icon(load_icon(
                                "mail-mark-important-symbolic",
                            )))
                            .push(
                                widget::text::body(
                                    item.tag_names
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
                        AppBookmarksMessage::EditBookmark(bookmark_account_id, item.to_owned()),
                    ),
                };
                let remove_bookmark_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("remove"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("remove")).font_size(12).on_press(
                        AppBookmarksMessage::DeleteBookmark(bookmark_account_id, item.to_owned()),
                    ),
                };
                let notes_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("notes"))
                        .font_size(12)
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("notes"))
                        .font_size(12)
                        .on_press(AppBookmarksMessage::ViewNotes(item.clone())),
                };
                let snapshot_button = match app_state {
                    ApplicationState::Refreshing => widget::button::link(fl!("snapshot"))
                        .spacing(spacing.space_xxxs)
                        .trailing_icon(true)
                        .font_size(12)
                        .icon_size(11)
                        .tooltip(item.web_archive_snapshot_url.clone())
                        .class(ButtonStyle::DisabledLink(false).into()),
                    _ => widget::button::link(fl!("snapshot"))
                        .spacing(spacing.space_xxxs)
                        .trailing_icon(true)
                        .font_size(12)
                        .icon_size(11)
                        .tooltip(item.web_archive_snapshot_url.clone())
                        .on_press(AppBookmarksMessage::OpenExternalURL(
                            item.web_archive_snapshot_url.clone(),
                        )),
                };
                let mut actions_row = widget::row::with_capacity(1)
                    .spacing(spacing.space_xxs)
                    .push(edit_bookmark_button)
                    .push(remove_bookmark_button);
                if !item.notes.is_empty() {
                    actions_row = actions_row.push(notes_button);
                }
                if !item.web_archive_snapshot_url.is_empty() {
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
                details_row = details_row
                    .push(widget::icon::icon(load_icon("accessories-clock-symbolic")).size(12))
                    .push(
                        widget::text(date_added.format("%a, %d %b %Y %H:%M:%S").to_string())
                            .size(12),
                    );
                if item.is_archived {
                    details_row = details_row
                        .push(widget::icon::icon(load_icon("mail-archive-symbolic")).size(12))
                        .push(widget::text(fl!("archived")).size(12));
                }
                if item.unread {
                    details_row = details_row
                        .push(
                            widget::icon::icon(load_icon("x-office-spreadsheet-symbolic")).size(12),
                        )
                        .push(widget::text(fl!("unread")).size(12));
                }
                if item.shared {
                    details_row = details_row
                        .push(widget::icon::icon(load_icon("emblem-shared-symbolic")).size(12))
                        .push(widget::text(fl!("shared")).size(12));
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

                let bookmark_container = widget::container(widget::column::with_children(columns));

                items = items.add(bookmark_container);
            }

            let refresh_button = if !self.query_placeholder.is_empty()
                || self.bookmarks.is_empty()
                || matches!(app_state, ApplicationState::Refreshing)
            {
                widget::button::standard(fl!("refresh"))
            } else {
                widget::button::standard(fl!("refresh"))
                    .on_press(AppBookmarksMessage::RefreshBookmarks)
            };
            let new_bookmark_button = match app_state {
                ApplicationState::Normal => widget::button::standard(fl!("add-bookmark"))
                    .on_press(AppBookmarksMessage::AddBookmark),
                _ => widget::button::standard(fl!("add-bookmark")),
            };

            let bookmarks_widget = widget::column::with_capacity(1)
                .spacing(spacing.space_xxs)
                .push(items)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill);

            let navigation_previous_button = widget::button::standard(fl!("previous"))
                .on_press_maybe(if bookmarks_cursor.current_page > 1 {
                    Some(AppBookmarksMessage::DecrementPageIndex)
                } else {
                    None
                })
                .leading_icon(load_icon("go-previous-symbolic"));
            let navigation_next_button = widget::button::standard(fl!("next"))
                .on_press_maybe(
                    if bookmarks_cursor.current_page < bookmarks_cursor.total_pages {
                        Some(AppBookmarksMessage::IncrementPageIndex)
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
                        bookmarks_cursor.current_page, bookmarks_cursor.total_pages
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
                        "bookmarks-with-count",
                        count = bookmarks_cursor.total_entries
                    )))
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ])
                    .push(
                        widget::search_input(fl!("search"), self.query_placeholder.clone())
                            .on_input(AppBookmarksMessage::SearchBookmarks)
                            .on_clear(AppBookmarksMessage::ClearSearch),
                    )
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
    pub fn update(&mut self, message: AppBookmarksMessage) -> Task<Message> {
        let mut commands = Vec::new();
        match message {
            AppBookmarksMessage::OpenAccountsPage => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::OpenAccountsPage)
                }));
            }
            AppBookmarksMessage::RefreshBookmarks => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::StartRefreshBookmarksForAllAccounts)
                }));
            }
            AppBookmarksMessage::AddBookmark => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::AddBookmarkForm)
                }));
            }
            AppBookmarksMessage::DeleteBookmark(account_id, bookmark) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::OpenRemoveBookmarkDialog(
                        account_id,
                        bookmark.clone(),
                    ))
                }));
            }
            AppBookmarksMessage::EditBookmark(account_id, bookmark) => {
                self.bookmark_placeholder = Some(bookmark.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::EditBookmark(account_id, bookmark.clone()))
                }));
            }
            AppBookmarksMessage::SearchBookmarks(query) => {
                self.query_placeholder.clone_from(&query);
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::SearchBookmarks(query.clone()))
                }));
            }
            AppBookmarksMessage::ClearSearch => {
                if !self.query_placeholder.is_empty() {
                    self.query_placeholder = String::new();
                    commands.push(Task::perform(async {}, |()| {
                        cosmic::app::Message::App(Message::SearchBookmarks(String::new()))
                    }));
                }
            }
            AppBookmarksMessage::OpenExternalURL(url) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::OpenExternalUrl(url.clone()))
                }));
            }
            AppBookmarksMessage::ViewNotes(bookmark) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::ViewBookmarkNotes(bookmark.clone()))
                }));
            }
            AppBookmarksMessage::EmptyMessage => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::Empty)
                }));
            }
            AppBookmarksMessage::IncrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::IncrementPageIndex("bookmarks".to_string()))
                }));
            }
            AppBookmarksMessage::DecrementPageIndex => {
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::DecrementPageIndex("bookmarks".to_string()))
                }));
            }
        }
        Task::batch(commands)
    }
}

#[allow(clippy::too_many_lines)]
pub fn new_bookmark<'a, 'b>(
    bookmark: Bookmark,
    accounts: &'b [Account],
    selected_account_index: usize,
) -> Element<'a, Message>
where
    'b: 'a,
{
    let spacing = theme::active().cosmic().spacing;
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let account_widget_title = widget::text::body(fl!("account"));
    let account_widget_dropdown =
        widget::dropdown(accounts, Some(selected_account_index), move |idx| {
            Message::AddBookmarkFormAccountIndex(idx)
        });
    let url_widget_title = widget::text::body(fl!("url"));
    let url_widget_text_input =
        widget::text_input("URL", bookmark.url.clone()).on_input(Message::SetBookmarkURL);
    let title_widget_title = widget::text::body(fl!("title"));
    let title_widget_text_input =
        widget::text_input("Title", bookmark.title.clone()).on_input(Message::SetBookmarkTitle);
    let description_widget_title = widget::text::body(fl!("description"));
    let description_widget_text_input =
        widget::text_input("Description", bookmark.description.clone())
            .on_input(Message::SetBookmarkDescription);
    let notes_widget_title = widget::text::body(fl!("notes"));
    let notes_widget_text_input = widget::text_input(fl!("notes"), bookmark.notes.clone())
        .on_input(Message::SetBookmarkNotes);
    let tags_widget_title = widget::text::body(fl!("tags"));
    let tags_widget_subtext = widget::text::caption(fl!("tags-subtext"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(Message::SetBookmarkTags);
    let archived_widget_checkbox = widget::checkbox(fl!("archived"), bookmark.is_archived)
        .on_toggle(Message::SetBookmarkArchived);
    let unread_widget_checkbox =
        widget::checkbox(fl!("unread"), bookmark.unread).on_toggle(Message::SetBookmarkUnread);
    let shared_widget_checkbox = if accounts[selected_account_index].clone().enable_sharing {
        widget::checkbox(fl!("shared"), bookmark.shared).on_toggle(Message::SetBookmarkShared)
    } else {
        widget::checkbox(fl!("shared-disabled"), false)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::AddBookmark(
            accounts[selected_account_index].clone(),
            bookmark,
        )),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("user-available-symbolic")))
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
                .push(widget::icon::icon(load_icon("web-browser-symbolic")))
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
                .push(widget::icon::icon(load_icon("insert-text-symbolic")))
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
                .push(widget::icon::icon(load_icon("text-x-generic-symbolic")))
                .push(description_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(description_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("x-office-document-symbolic")))
                .push(notes_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(notes_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon(
                    "mail-mark-important-symbolic",
                )))
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
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}

#[allow(clippy::too_many_lines)]
pub fn edit_bookmark<'a, 'b>(bookmark: Bookmark, account: &'b Account) -> Element<'a, Message>
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
        widget::text_input("URL", bookmark.url.clone()).on_input(Message::SetBookmarkURL);
    let title_widget_title = widget::text::body(fl!("title"));
    let title_widget_text_input =
        widget::text_input("Title", bookmark.title.clone()).on_input(Message::SetBookmarkTitle);
    let description_widget_title = widget::text::body(fl!("description"));
    let description_widget_text_input =
        widget::text_input("Description", bookmark.description.clone())
            .on_input(Message::SetBookmarkDescription);
    let notes_widget_title = widget::text::body(fl!("notes"));
    let notes_widget_text_input = widget::text_input(fl!("notes"), bookmark.notes.clone())
        .on_input(Message::SetBookmarkNotes);
    let tags_widget_title = widget::text::body(fl!("tags"));
    let tags_widget_subtext = widget::text::caption(fl!("tags-subtext"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(Message::SetBookmarkTags);
    let archived_widget_checkbox = widget::checkbox(fl!("archived"), bookmark.is_archived)
        .on_toggle(Message::SetBookmarkArchived);
    let unread_widget_checkbox =
        widget::checkbox(fl!("unread"), bookmark.unread).on_toggle(Message::SetBookmarkUnread);
    let shared_widget_checkbox = if account.clone().enable_sharing {
        widget::checkbox(fl!("shared"), bookmark.shared).on_toggle(Message::SetBookmarkShared)
    } else {
        widget::checkbox(fl!("shared-disabled"), false)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save"))
            .on_press(Message::UpdateBookmark(account.clone(), bookmark)),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    widget::column()
        .spacing(space_xxs)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("user-available-symbolic")))
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
                .push(widget::icon::icon(load_icon("web-browser-symbolic")))
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
                .push(widget::icon::icon(load_icon("insert-text-symbolic")))
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
                .push(widget::icon::icon(load_icon("text-x-generic-symbolic")))
                .push(description_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Start),
        )
        .push(description_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon("x-office-document-symbolic")))
                .push(notes_widget_title)
                .padding([
                    spacing.space_xxxs,
                    spacing.space_xxs,
                    spacing.space_none,
                    spacing.space_none,
                ])
                .align_y(Alignment::Center),
        )
        .push(notes_widget_text_input)
        .push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(widget::icon::icon(load_icon(
                    "mail-mark-important-symbolic",
                )))
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
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::Space::new(0, 5))
        .push(buttons_widget_container)
        .into()
}

pub fn view_notes<'a>(bookmark: Bookmark) -> Element<'a, Message> {
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

    widget::column()
        .spacing(space_xxs)
        .push(widget::text::body(bookmark.notes))
        .into()
}
