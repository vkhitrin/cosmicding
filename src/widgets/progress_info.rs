use cosmic::{
    iced::{Alignment, Length},
    theme, widget, Apply, Element,
};

#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub total: usize,
    pub current: usize,
    pub label: String,
    pub cancellable: bool,
}

pub fn operation_progress_widget<'a, Message: 'a + 'static + Clone>(
    progress: &ProgressInfo,
    on_cancel: Option<Message>,
) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let progress_text = format!(
        "{}: {} / {}",
        progress.label,
        progress.current + 1,
        progress.total
    );
    let progress_percentage = ((progress.current + 1) as f32 / progress.total as f32) * 100.0;

    let mut row = widget::row::with_capacity(3)
        .spacing(spacing.space_xxs)
        .padding([
            spacing.space_none,
            spacing.space_none,
            spacing.space_xxs,
            spacing.space_none,
        ])
        .align_y(Alignment::Center)
        .push(widget::text::body(progress_text).size(13))
        .push(widget::horizontal_space());

    if progress.cancellable {
        if let Some(cancel_action) = on_cancel {
            row = row.push(
                widget::button::text("Cancel")
                    .on_press(cancel_action)
                    .class(cosmic::theme::Button::Destructive)
                    .padding([spacing.space_xxxs, spacing.space_xxs])
                    .font_size(12),
            );
        }
    }

    widget::column::with_capacity(2)
        .spacing(spacing.space_xxxs)
        .push(row)
        .push(widget::progress_bar(0.0..=100.0, progress_percentage).height(Length::Fixed(4.0)))
        .apply(widget::container)
        .class(cosmic::theme::Container::Background)
        .into()
}
