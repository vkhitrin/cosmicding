use crate::app::Message;
use crate::fl;
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;
use chrono::{DateTime, Local};
use cosmic::iced::Length;
use cosmic::{
    app::command::Task,
    iced::{self, Alignment},
    widget::{self},
    Apply, Element,
};
use cosmic::{cosmic_theme, theme};
use iced::alignment::{Horizontal, Vertical};

#[derive(Debug, Clone, Default)]
pub struct PageBookmarksView {
    pub accounts: Vec<Account>,
    pub bookmarks: Vec<Bookmark>,
    account_placeholder: Option<Account>,
    bookmark_placeholder: Option<Bookmark>,
    query_placeholder: String,
}

#[derive(Debug, Clone)]
pub enum AppBookmarksMessage {
    ClearSearch,
    DeleteBookmark(Account, Bookmark),
    EditBookmark(Account, Bookmark),
    AddBookmark,
    OpenAccountsPage,
    OpenExternalURL(String),
    RefreshBookmarks,
    SearchBookmarks(String),
    ViewNotes(Bookmark),
    EmptyMessage,
}

impl PageBookmarksView {
    #[allow(clippy::too_many_lines)]
    pub fn view(&self) -> Element<'_, AppBookmarksMessage> {
        let spacing = theme::active().cosmic().spacing;
        if self.accounts.is_empty() {
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("web-browser-symbolic")
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

            for item in &self.bookmarks {
                let derived_account = self
                    .accounts
                    .iter()
                    .find(|&account| account.id == item.user_account_id)
                    .unwrap();
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
                        .push(widget::icon::from_name("web-browser-symbolic"))
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
                            .push(widget::icon::from_name("emblem-documents-symbolic"))
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
                            .push(widget::icon::from_name("mail-mark-important-symbolic"))
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
                let mut actions_row = widget::row::with_capacity(1)
                    .spacing(spacing.space_xxs)
                    .push(widget::button::link(fl!("edit")).font_size(12).on_press(
                        AppBookmarksMessage::EditBookmark(
                            derived_account.to_owned(),
                            item.to_owned(),
                        ),
                    ))
                    .push(widget::button::link(fl!("remove")).font_size(12).on_press(
                        AppBookmarksMessage::DeleteBookmark(
                            derived_account.to_owned(),
                            item.to_owned(),
                        ),
                    ));
                if !item.notes.is_empty() {
                    actions_row = actions_row.push(
                        widget::button::link(fl!("notes"))
                            .font_size(12)
                            .on_press(AppBookmarksMessage::ViewNotes(item.clone())),
                    );
                }
                if !item.web_archive_snapshot_url.is_empty() {
                    actions_row = actions_row.push(
                        widget::button::link(fl!("snapshot"))
                            .spacing(spacing.space_xxxs)
                            .trailing_icon(true)
                            .font_size(12)
                            .icon_size(11)
                            .tooltip(item.web_archive_snapshot_url.clone())
                            .on_press(AppBookmarksMessage::OpenExternalURL(
                                item.web_archive_snapshot_url.clone(),
                            )),
                    );
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
                    .push(widget::icon::from_name("accessories-clock-symbolic").size(12))
                    .push(widget::text(date_added.to_rfc2822()).size(12));
                if item.is_archived {
                    details_row = details_row
                        .push(widget::icon::from_name("mail-archive-symbolic").size(12))
                        .push(widget::text(fl!("archived")).size(12));
                }
                if item.unread {
                    details_row = details_row
                        .push(widget::icon::from_name("x-office-spreadsheet-symbolic").size(12))
                        .push(widget::text(fl!("unread")).size(12));
                }
                if item.shared {
                    details_row = details_row
                        .push(widget::icon::from_name("emblem-shared-symbolic").size(12))
                        .push(widget::text(fl!("shared")).size(12));
                }
                columns.push(
                    details_row
                        .push(widget::horizontal_space())
                        .push(widget::icon::from_name("user-available-symbolic").size(12))
                        .push(widget::text(derived_account.display_name.clone()).size(12))
                        .align_y(iced::alignment::Vertical::Center)
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

            let bookmarks_widget = widget::column::with_capacity(1)
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
                        "bookmarks-with-count",
                        count = self.bookmarks.len()
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
                    .push(
                        if !self.query_placeholder.is_empty() || self.bookmarks.is_empty() {
                            widget::button::standard(fl!("refresh"))
                        } else {
                            widget::button::standard(fl!("refresh"))
                                .on_press(AppBookmarksMessage::RefreshBookmarks)
                        },
                    )
                    .push(
                        widget::button::standard(fl!("add-bookmark"))
                            .on_press(AppBookmarksMessage::AddBookmark),
                    )
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into()])
                .push(bookmarks_widget),
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
            AppBookmarksMessage::DeleteBookmark(account, bookmark) => {
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::OpenRemoveBookmarkDialog(
                        account.clone(),
                        bookmark.clone(),
                    ))
                }));
            }
            AppBookmarksMessage::EditBookmark(account, bookmark) => {
                self.account_placeholder = Some(account.clone());
                self.bookmark_placeholder = Some(bookmark.clone());
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::EditBookmark(
                        account.clone(),
                        bookmark.clone(),
                    ))
                }));
            }
            AppBookmarksMessage::SearchBookmarks(query) => {
                self.query_placeholder.clone_from(&query);
                commands.push(Task::perform(async {}, move |()| {
                    cosmic::app::Message::App(Message::SearchBookmarks(query.clone()))
                }));
            }
            AppBookmarksMessage::ClearSearch => {
                self.query_placeholder = String::new();
                commands.push(Task::perform(async {}, |()| {
                    cosmic::app::Message::App(Message::LoadBookmarks)
                }));
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
        }
        Task::batch(commands)
    }
}

pub fn new_bookmark<'a, 'b>(
    bookmark: Bookmark,
    accounts: &'b [Account],
    selected_account_index: usize,
) -> Element<'a, Message>
where
    'b: 'a,
{
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
        widget::checkbox(fl!("shared-disabled"), false).on_toggle(|_| Message::Empty)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save")).on_press(Message::AddBookmark(
            accounts[selected_account_index].clone(),
            bookmark,
        )),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    widget::column()
        .spacing(space_xxs)
        .push(account_widget_title)
        .push(account_widget_dropdown)
        .push(url_widget_title)
        .push(url_widget_text_input)
        .push(title_widget_title)
        .push(title_widget_text_input)
        .push(description_widget_title)
        .push(description_widget_text_input)
        .push(notes_widget_title)
        .push(notes_widget_text_input)
        .push(tags_widget_title)
        .push(tags_widget_subtext)
        .push(tags_widget_text_input)
        .push(widget::Space::new(0, 10))
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::Space::new(0, 10))
        .push(buttons_widget_container)
        .into()
}

pub fn edit_bookmark<'a, 'b>(bookmark: Bookmark, accounts: &'b [Account]) -> Element<'a, Message>
where
    'b: 'a,
{
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let account = accounts
        .iter()
        .find(|account| account.id == bookmark.user_account_id)
        .cloned();
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
    let notes_widget_text_input =
        widget::text_input("notes", bookmark.notes.clone()).on_input(Message::SetBookmarkNotes);
    let tags_widget_title = widget::text::body(fl!("tags"));
    let tags_widget_subtext = widget::text::caption(fl!("tags-subtext"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(Message::SetBookmarkTags);
    let archived_widget_checkbox = widget::checkbox(fl!("archived"), bookmark.is_archived)
        .on_toggle(Message::SetBookmarkArchived);
    let unread_widget_checkbox =
        widget::checkbox(fl!("unread"), bookmark.unread).on_toggle(Message::SetBookmarkUnread);
    let shared_widget_checkbox = if account.clone().unwrap().enable_sharing {
        widget::checkbox(fl!("shared"), bookmark.shared).on_toggle(Message::SetBookmarkShared)
    } else {
        widget::checkbox(fl!("shared-disabled"), false).on_toggle(|_| Message::Empty)
    };
    let buttons_widget_container = widget::container(
        widget::button::standard(fl!("save"))
            .on_press(Message::UpdateBookmark(account.unwrap(), bookmark)),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    widget::column()
        .spacing(space_xxs)
        .push(url_widget_title)
        .push(url_widget_text_input)
        .push(title_widget_title)
        .push(title_widget_text_input)
        .push(description_widget_title)
        .push(description_widget_text_input)
        .push(notes_widget_title)
        .push(notes_widget_text_input)
        .push(tags_widget_title)
        .push(tags_widget_subtext)
        .push(tags_widget_text_input)
        .push(widget::Space::new(0, 10))
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::Space::new(0, 10))
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
