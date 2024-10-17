use crate::app::Message;
use crate::fl;
use crate::models::account::Account;
use crate::models::bookmarks::Bookmark;
use cosmic::iced::Length;
use cosmic::{cosmic_theme, theme};
use cosmic::{
    iced::{self, Alignment},
    widget::{self},
    Apply, Command, Element,
};
use iced::alignment::{Horizontal, Vertical};

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct BookmarksView {
    pub accounts: Vec<Account>,
    pub bookmarks: Vec<Bookmark>,
    account_placeholder: Option<Account>,
    bookmark_placeholder: Option<Bookmark>,
    query_placeholder: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub enum BookmarksMessage {
    ClearSearch,
    DeleteBookmark(Account, Bookmark),
    EditBookmark(Account, Bookmark),
    NewBookmark,
    OpenAccountsPage,
    OpenExternalURL(String),
    RefreshBookmarks,
    SearchBookmarks(String),
    ViewNotes(Bookmark),
}

impl Default for BookmarksView {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            bookmarks: Vec::new(),
            account_placeholder: None,
            bookmark_placeholder: None,
            query_placeholder: Cow::Owned("".to_string()),
        }
    }
}

impl BookmarksView {
    pub fn view<'a>(&'a self) -> Element<'a, BookmarksMessage> {
        let spacing = theme::active().cosmic().spacing;
        let mut action_button = widget::button::text(fl!("add-bookmark"))
            .style(widget::button::Style::Standard)
            .on_press(BookmarksMessage::NewBookmark);
        let mut page_title_text = widget::text::title3(fl!("no-bookmarks"));
        if self.bookmarks.is_empty() && self.query_placeholder.is_empty() {
            if self.accounts.is_empty() {
                page_title_text = widget::text::title3(fl!("no-accounts"));
                action_button = widget::button::text(fl!("open-accounts-page"))
                    .style(widget::button::Style::Standard)
                    .on_press(BookmarksMessage::OpenAccountsPage);
            }
            let container = widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("web-browser-symbolic")
                        .size(64)
                        .into(),
                    page_title_text.into(),
                    action_button.into(),
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

            for item in self.bookmarks.iter() {
                let derived_account = self
                    .accounts
                    .iter()
                    .find(|&account| account.id == item.user_account_id)
                    .unwrap();
                let mut columns = Vec::new();
                // Mandatory first row - title
                columns.push(
                    widget::row::with_capacity(1)
                        .spacing(spacing.space_xxs)
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_none,
                        ])
                        .push(
                            widget::button::link(item.title.clone())
                                .on_press(BookmarksMessage::OpenExternalURL(item.url.clone())),
                        )
                        .into(),
                );
                // Optional second row - description
                if !item.description.is_empty() {
                    columns.push(
                        widget::row::with_capacity(1)
                            .spacing(spacing.space_xxs)
                            .padding([
                                spacing.space_xxxs,
                                spacing.space_xxs,
                                spacing.space_xxs,
                                spacing.space_xxxs,
                            ])
                            .push(widget::text(item.description.clone()))
                            .into(),
                    );
                }
                // Optional third row - tags
                if item.tag_names.len() > 0 {
                    columns.push(
                        widget::row::with_capacity(1)
                            .spacing(spacing.space_xxs)
                            .padding([
                                spacing.space_xxxs,
                                spacing.space_xxs,
                                spacing.space_xxs,
                                spacing.space_xxxs,
                            ])
                            .push(widget::text::body(
                                item.tag_names
                                    .iter()
                                    .map(|s| format!("#{}", s))
                                    .collect::<Vec<String>>()
                                    .join(" "),
                            ))
                            .into(),
                    );
                }
                // Mandatory fourth row - actions
                let mut actions_row = widget::row::with_capacity(1)
                    .spacing(spacing.space_xxs)
                    .push(widget::button::link(fl!("edit")).on_press(
                        BookmarksMessage::EditBookmark(derived_account.to_owned(), item.to_owned()),
                    ))
                    .push(widget::button::link(fl!("remove")).on_press(
                        BookmarksMessage::DeleteBookmark(
                            derived_account.to_owned(),
                            item.to_owned(),
                        ),
                    ));
                if !item.notes.is_empty() {
                    actions_row = actions_row.push(
                        widget::button::link(fl!("notes"))
                            .on_press(BookmarksMessage::ViewNotes(item.clone())),
                    );
                }
                columns.push(
                    actions_row
                        .push(widget::horizontal_space(Length::Fill))
                        .push(widget::text(
                            "[".to_string() + &derived_account.display_name + "]",
                        ))
                        .padding([
                            spacing.space_xxxs,
                            spacing.space_xxs,
                            spacing.space_xxs,
                            spacing.space_none,
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
                widget::column::with_children(vec![widget::row::with_capacity(3)
                    .align_items(Alignment::Center)
                    .push(widget::text::title3(fl!("bookmarks")))
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_none,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ])
                    .push(
                        widget::search_input(fl!("search"), self.query_placeholder.as_ref())
                            .on_input(BookmarksMessage::SearchBookmarks)
                            .on_clear(BookmarksMessage::ClearSearch),
                    )
                    .push(
                        widget::button::text(fl!("refresh"))
                            .on_press(BookmarksMessage::RefreshBookmarks)
                            .style(theme::Button::Standard),
                    )
                    .push(
                        widget::button::text(fl!("add-bookmark"))
                            .on_press(BookmarksMessage::NewBookmark)
                            .style(theme::Button::Standard),
                    )
                    .width(Length::Fill)
                    .apply(widget::container)
                    .into()])
                .push(bookmarks_widget),
            )
            .into()
        }
    }
    pub fn update(&mut self, message: BookmarksMessage) -> Command<Message> {
        let mut commands = Vec::new();
        match message {
            BookmarksMessage::OpenAccountsPage => {
                commands.push(Command::perform(async {}, |_| Message::OpenAccountsPage));
            }
            BookmarksMessage::RefreshBookmarks => {
                commands.push(Command::perform(async {}, |_| Message::RefreshAllBookmarks));
            }
            BookmarksMessage::NewBookmark => {
                commands.push(Command::perform(async {}, |_| Message::AddBookmarkForm));
            }
            BookmarksMessage::DeleteBookmark(account, bookmark) => {
                commands.push(Command::perform(async {}, move |_| {
                    Message::OpenRemoveBookmarkDialog(account, bookmark)
                }));
            }
            BookmarksMessage::EditBookmark(account, bookmark) => {
                self.account_placeholder = Some(account.clone());
                self.bookmark_placeholder = Some(bookmark.clone());
                commands.push(Command::perform(async {}, move |_| {
                    Message::EditBookmark(account, bookmark)
                }));
            }
            BookmarksMessage::SearchBookmarks(query) => {
                self.query_placeholder = Cow::Owned(query.to_string());
                commands.push(Command::perform(async {}, move |_| {
                    Message::SearchBookmarks(query)
                }));
            }
            BookmarksMessage::ClearSearch => {
                self.query_placeholder = Cow::Owned("".to_string());
                commands.push(Command::perform(async {}, |_| Message::RefreshAllBookmarks));
            }
            BookmarksMessage::OpenExternalURL(url) => {
                commands.push(Command::perform(async {}, |_| {
                    Message::OpenExternalUrl(url)
                }));
            }
            BookmarksMessage::ViewNotes(bookmark) => {
                commands.push(Command::perform(async {}, |_| {
                    Message::ViewBookmarkNotes(bookmark)
                }));
            }
        }
        Command::batch(commands)
    }
}

pub fn new_bookmark<'a, 'b>(
    bookmark: Bookmark,
    accounts: &'b Vec<Account>,
    selected_account: usize,
) -> Element<'a, Message>
where
    'b: 'a,
{
    let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
    let account_widget_title = widget::text::body(fl!("account"));
    let account_widget_dropdown = widget::dropdown(&accounts, Some(selected_account), move |idx| {
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
    let tags_widget_subtext = widget::text::caption(fl!("tags-helper"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(Message::SetBookmarkTags);
    let archived_widget_checkbox = widget::checkbox(
        fl!("archived"),
        bookmark.is_archived,
        Message::SetBookmarkArchived,
    );
    let unread_widget_checkbox =
        widget::checkbox(fl!("unread"), bookmark.unread, Message::SetBookmarkUnread);
    let shared_widget_checkbox =
        widget::checkbox(fl!("shared"), bookmark.shared, Message::SetBookmarkShared);
    let buttons_widget_container = widget::container(
        widget::button::text(fl!("save"))
            .style(widget::button::Style::Standard)
            .on_press(Message::AddBookmark(
                accounts[selected_account].clone(),
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
        .push(widget::vertical_space(Length::from(10)))
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::vertical_space(Length::from(10)))
        .push(buttons_widget_container)
        .into()
}

pub fn edit_bookmark<'a, 'b>(bookmark: Bookmark, accounts: &'b Vec<Account>) -> Element<'a, Message>
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
    let tags_widget_subtext = widget::text::caption(fl!("tags-helper"));
    let tags_widget_text_input = widget::text_input("Tags", bookmark.tag_names.join(" ").clone())
        .on_input(Message::SetBookmarkTags);
    let archived_widget_checkbox = widget::checkbox(
        fl!("archived"),
        bookmark.is_archived,
        Message::SetBookmarkArchived,
    );
    let unread_widget_checkbox =
        widget::checkbox(fl!("unread"), bookmark.unread, Message::SetBookmarkUnread);
    let shared_widget_checkbox =
        widget::checkbox(fl!("shared"), bookmark.shared, Message::SetBookmarkShared);
    let buttons_widget_container = widget::container(
        widget::button::text(fl!("save"))
            .style(widget::button::Style::Standard)
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
        .push(widget::vertical_space(Length::from(10)))
        .push(archived_widget_checkbox)
        .push(unread_widget_checkbox)
        .push(shared_widget_checkbox)
        .push(widget::vertical_space(Length::from(10)))
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
