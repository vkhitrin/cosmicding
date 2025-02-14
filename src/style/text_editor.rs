pub fn text_editor_class(
    theme: &cosmic::Theme,
    status: cosmic::widget::text_editor::Status,
) -> cosmic::iced_widget::text_editor::Style {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: cosmic::iced::Color = container.component.base.into();
    background.a = 0.25;
    let selection = cosmic.accent.base.into();
    let value = cosmic.palette.neutral_9.into();
    let mut placeholder = cosmic.palette.neutral_9;
    placeholder.alpha = 0.7;
    let placeholder = placeholder.into();
    let icon = cosmic.background.on.into();

    match status {
        cosmic::iced_widget::text_editor::Status::Active
        | cosmic::iced_widget::text_editor::Status::Disabled => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: container.component.divider.into(),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
        cosmic::iced_widget::text_editor::Status::Hovered
        | cosmic::iced_widget::text_editor::Status::Focused => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: cosmic::iced::Color::from(cosmic.accent.base),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
    }
}
