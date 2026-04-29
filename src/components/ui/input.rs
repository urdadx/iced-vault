use iced::widget::{TextInput, text_input};
use iced::{Background, Border, Color, Length, Theme};

use crate::components::ui::{THEME_CORNER_RADIUS, THEME_FONT_SIZE, darken};

#[allow(dead_code)]
pub fn input<'a, Message: Clone>(placeholder: &str, value: &str) -> TextInput<'a, Message> {
    text_input(placeholder, value)
        .padding([8.0, 12.0])
        .size(THEME_FONT_SIZE)
        .width(Length::Fill)
        .style(input_style)
}

#[allow(dead_code)]
pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    let border = darken(palette.background, 0.15);
    let placeholder = palette.text.scale_alpha(0.55);

    match status {
        text_input::Status::Active => style(
            palette.background,
            border,
            1.0,
            palette.text,
            placeholder,
            placeholder,
            palette.primary.scale_alpha(0.2),
        ),
        text_input::Status::Hovered => style(
            darken(palette.background, 0.02),
            darken(border, 0.05),
            1.0,
            palette.text,
            placeholder,
            palette.text.scale_alpha(0.7),
            palette.primary.scale_alpha(0.22),
        ),
        text_input::Status::Focused { is_hovered } => style(
            if is_hovered {
                darken(palette.background, 0.02)
            } else {
                palette.background
            },
            palette.primary.scale_alpha(0.75),
            2.0,
            palette.text,
            placeholder,
            palette.primary,
            palette.primary.scale_alpha(0.28),
        ),
        text_input::Status::Disabled => style(
            darken(palette.background, 0.01),
            border.scale_alpha(0.5),
            1.0,
            palette.text.scale_alpha(0.5),
            placeholder.scale_alpha(0.8),
            placeholder.scale_alpha(0.8),
            palette.primary.scale_alpha(0.12),
        ),
    }
}

fn style(
    background: Color,
    border_color: Color,
    border_width: f32,
    value: Color,
    placeholder: Color,
    icon: Color,
    selection: Color,
) -> text_input::Style {
    text_input::Style {
        background: Background::Color(background),
        border: Border {
            color: border_color,
            width: border_width,
            radius: THEME_CORNER_RADIUS.into(),
        },
        icon,
        placeholder,
        value,
        selection,
    }
}
