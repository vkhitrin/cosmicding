use cosmic::{
    cosmic_theme::Component,
    iced::{Background, Color},
    theme::{Button, TRANSPARENT_COMPONENT},
    widget::{self},
};

pub fn disabled_link_button() -> Button {
    Button::Custom {
        active: Box::new(move |_, _| {
            appearance(false, false, false, &Button::Link, |component| {
                let text_color = Some(component.on.into());
                (component.hover.into(), text_color, text_color)
            })
        }),
        disabled: Box::new(move |_| {
            appearance(false, false, true, &Button::Link, |component| {
                let mut background = Color::from(component.base);
                background.a *= 0.5;
                (
                    background,
                    Some(component.on_disabled.into()),
                    Some(component.on_disabled.into()),
                )
            })
        }),
        hovered: Box::new(move |_, _| {
            appearance(false, false, false, &Button::Link, |component| {
                let text_color = Some(component.on.into());

                (component.hover.into(), text_color, text_color)
            })
        }),
        pressed: Box::new(move |_, _| {
            appearance(false, false, false, &Button::Link, |component| {
                let text_color = Some(component.on.into());

                (component.pressed.into(), text_color, text_color)
            })
        }),
    }
}

pub fn appearance(
    focused: bool,
    selected: bool,
    disabled: bool,
    style: &Button,
    color: impl Fn(&Component) -> (Color, Option<Color>, Option<Color>),
) -> widget::button::Style {
    let cosmic = cosmic::theme::active().cosmic().clone();
    let mut corner_radii = &cosmic.corner_radii.radius_xl;
    let mut appearance = widget::button::Style::new();

    match style {
        Button::Standard
        | Button::Text
        | Button::Suggested
        | Button::Destructive
        | Button::Transparent => {
            let style_component = match style {
                Button::Standard => &cosmic.button,
                Button::Text => &cosmic.text_button,
                Button::Suggested => &cosmic.accent_button,
                Button::Destructive => &cosmic.destructive_button,
                Button::Transparent => &TRANSPARENT_COMPONENT,
                _ => return appearance,
            };

            let (background, text, icon) = color(style_component);
            appearance.background = Some(Background::Color(background));
            if !matches!(style, Button::Standard) {
                appearance.text_color = text;
                appearance.icon_color = icon;
            }
        }

        Button::Icon | Button::IconVertical | Button::HeaderBar => {
            if matches!(style, Button::IconVertical) {
                corner_radii = &cosmic.corner_radii.radius_m;
                if selected {
                    appearance.overlay = Some(Background::Color(Color::from(
                        cosmic.icon_button.selected_state_color(),
                    )));
                }
            }

            let (background, text, icon) = color(&cosmic.icon_button);
            appearance.background = Some(Background::Color(background));
            // Only override icon button colors when it is disabled
            appearance.icon_color = if disabled { icon } else { None };
            appearance.text_color = if disabled { text } else { None };
        }

        Button::Image => {
            appearance.background = None;
            appearance.text_color = Some(cosmic.accent.base.into());
            appearance.icon_color = Some(cosmic.accent.base.into());

            corner_radii = &cosmic.corner_radii.radius_s;
            appearance.border_radius = (*corner_radii).into();

            if focused || selected {
                appearance.border_width = 2.0;
                appearance.border_color = cosmic.accent.base.into();
            }

            return appearance;
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent.on_disabled.into());
            appearance.text_color = Some(cosmic.accent.on_disabled.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::Custom { .. } => (),
        Button::AppletMenu => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }
        Button::AppletIcon => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
        }
        Button::MenuFolder => {
            // Menu folders cannot be disabled, ignore customized icon and text color
            let component = &cosmic.background.component;
            let (background, _, _) = color(component);
            appearance.background = Some(Background::Color(background));
            appearance.icon_color = Some(component.on.into());
            appearance.text_color = Some(component.on.into());
            corner_radii = &cosmic.corner_radii.radius_s;
        }
        Button::MenuItem => {
            let (background, text, icon) = color(&cosmic.background.component);
            appearance.background = Some(Background::Color(background));
            appearance.icon_color = icon;
            appearance.text_color = text;
            corner_radii = &cosmic.corner_radii.radius_s;
        }
        Button::MenuRoot => {
            appearance.background = None;
            appearance.icon_color = None;
            appearance.text_color = None;
        }
    }

    appearance.border_radius = (*corner_radii).into();

    if focused {
        appearance.outline_width = 1.0;
        appearance.outline_color = cosmic.accent.base.into();
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }

    appearance
}
