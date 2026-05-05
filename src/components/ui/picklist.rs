use iced::widget::pick_list;
use iced::{Background, Border, Color, Theme};

use crate::components::ui::{THEME_CORNER_RADIUS, darken};

#[allow(dead_code)]
pub fn s_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();
    let border = darken(palette.background, 0.15);
    let placeholder = palette.text.scale_alpha(0.55);

    match status {
        pick_list::Status::Active => style(
            palette.background,
            border,
            1.0,
            palette.text,
            placeholder,
            placeholder,
        ),
        pick_list::Status::Hovered => style(
            darken(palette.background, 0.02),
            darken(border, 0.05),
            1.0,
            palette.text,
            placeholder,
            palette.text.scale_alpha(0.7),
        ),
        pick_list::Status::Opened { .. } => style(
            palette.background,
            palette.primary.scale_alpha(0.75),
            2.0,
            palette.text,
            placeholder,
            palette.primary,
        ),
    }
}

fn style(
    background: Color,
    border_color: Color,
    border_width: f32,
    text_color: Color,
    placeholder_color: Color,
    handle_color: Color,
) -> pick_list::Style {
    pick_list::Style {
        text_color,
        placeholder_color,
        handle_color,
        background: Background::Color(background),
        border: Border {
            color: border_color,
            width: border_width,
            radius: THEME_CORNER_RADIUS.into(),
        },
    }
}
