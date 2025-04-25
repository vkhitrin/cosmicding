use cosmic::{iced, widget};

pub enum ButtonStyle {
    DisabledLink(bool),
}

impl From<ButtonStyle> for cosmic::style::Button {
    fn from(style: ButtonStyle) -> Self {
        match style {
            ButtonStyle::DisabledLink(is_selected) => Self::Custom {
                active: Box::new(move |_, theme| {
                    let cosmic = theme.cosmic();
                    let mut appearance = widget::button::Style::new();
                    if is_selected {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.accent.base.into()));
                        appearance.text_color = Some(cosmic.accent.on.into());
                    }
                    appearance.border_radius = cosmic.radius_s().into();
                    appearance
                }),

                disabled: Box::new(move |theme| {
                    let cosmic = theme.cosmic();
                    let mut appearance = widget::button::Style::new();
                    if is_selected {
                        appearance.background = Some(iced::Background::Color(
                            cosmic.background.component.base.into(),
                        ));
                        appearance.text_color = Some(cosmic.accent.on.into());
                    } else {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.background.base.into()));
                        appearance.text_color = Some(cosmic.button.on_disabled.into());
                    }

                    appearance
                }),
                hovered: Box::new(move |_, theme| {
                    let cosmic = theme.cosmic();
                    let mut appearance = widget::button::Style::new();
                    if is_selected {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.accent.hover.into()));
                        appearance.text_color = Some(cosmic.accent.on.into());
                    } else {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.button.hover.into()));
                        appearance.text_color = Some(cosmic.button.on.into());
                    }
                    appearance.border_radius = cosmic.radius_s().into();
                    appearance
                }),
                pressed: Box::new(move |_, theme| {
                    let cosmic = theme.cosmic();
                    let mut appearance = widget::button::Style::new();
                    if is_selected {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.accent.pressed.into()));
                        appearance.text_color = Some(cosmic.accent.on.into());
                    } else {
                        appearance.background =
                            Some(iced::Background::Color(cosmic.button.pressed.into()));
                        appearance.text_color = Some(cosmic.button.on.into());
                    }
                    appearance.border_radius = cosmic.radius_s().into();
                    appearance
                }),
            },
        }
    }
}
