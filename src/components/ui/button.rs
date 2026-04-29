use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme};

use crate::components::ui::{THEME_CORNER_RADIUS, darken, lighten};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Outline,
    Secondary,
    Ghost,
    Destructive,
    Link,
}

// Iced keeps layout outside of the style function, so size helpers expose
// padding and text sizing separately from visual styling.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

#[allow(dead_code)]
impl ButtonSize {
    pub const fn padding(self) -> [f32; 2] {
        match self {
            Self::Default => [8.0, 10.0],
            Self::Xs => [4.0, 8.0],
            Self::Sm => [6.0, 10.0],
            Self::Lg => [12.0, 12.0],
            Self::Icon => [8.0, 8.0],
            Self::IconXs => [4.0, 4.0],
            Self::IconSm => [6.0, 6.0],
            Self::IconLg => [10.0, 10.0],
        }
    }

    pub const fn gap(self) -> f32 {
        match self {
            Self::Xs | Self::Sm | Self::IconXs | Self::IconSm => 4.0,
            _ => 6.0,
        }
    }

    pub const fn text_size(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 11.0,
            Self::Lg | Self::IconLg => 14.0,
            _ => 12.0,
        }
    }

    pub const fn square_size(self) -> Option<f32> {
        match self {
            Self::Icon => Some(36.0),
            Self::IconXs => Some(24.0),
            Self::IconSm => Some(32.0),
            Self::IconLg => Some(40.0),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub fn button_style(
    variant: ButtonVariant,
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme, status| variant_style(theme, status, variant)
}

#[allow(dead_code)]
pub fn default_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Default)
}

#[allow(dead_code)]
pub fn outline_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Outline)
}

#[allow(dead_code)]
pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Secondary)
}

#[allow(dead_code)]
pub fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Ghost)
}

#[allow(dead_code)]
pub fn destructive_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Destructive)
}

#[allow(dead_code)]
pub fn link_button(theme: &Theme, status: button::Status) -> button::Style {
    variant_style(theme, status, ButtonVariant::Link)
}

fn variant_style(theme: &Theme, status: button::Status, variant: ButtonVariant) -> button::Style {
    let palette = theme.palette();

    match variant {
        ButtonVariant::Default => match status {
            button::Status::Active => {
                style(Some(palette.primary), palette.text, Color::TRANSPARENT, 0.0)
            }
            button::Status::Hovered => style(
                Some(darken(palette.primary, 0.08)),
                palette.text,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Pressed => style(
                Some(darken(palette.primary, 0.14)),
                palette.text,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Disabled => style(
                Some(palette.primary.scale_alpha(0.2)),
                palette.text.scale_alpha(0.5),
                Color::TRANSPARENT,
                0.0,
            ),
        },
        ButtonVariant::Outline => {
            let border_color = darken(palette.background, 0.15);

            match status {
                button::Status::Active => style(
                    Some(darken(palette.background, 0.02)),
                    palette.text,
                    border_color,
                    1.0,
                ),
                button::Status::Hovered => style(
                    Some(darken(palette.background, 0.06)),
                    palette.text,
                    darken(border_color, 0.04),
                    1.0,
                ),
                button::Status::Pressed => style(
                    Some(darken(palette.background, 0.1)),
                    darken(palette.text, 0.05),
                    border_color,
                    1.0,
                ),
                button::Status::Disabled => style(
                    Some(palette.background),
                    palette.text.scale_alpha(0.5),
                    border_color.scale_alpha(0.5),
                    1.0,
                ),
            }
        }
        ButtonVariant::Secondary => match status {
            button::Status::Active => {
                style(Some(palette.success), palette.text, Color::TRANSPARENT, 0.0)
            }
            button::Status::Hovered => style(
                Some(darken(palette.success, 0.08)),
                palette.text,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Pressed => style(
                Some(darken(palette.success, 0.14)),
                palette.text,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Disabled => style(
                Some(palette.success.scale_alpha(0.2)),
                palette.text.scale_alpha(0.5),
                Color::TRANSPARENT,
                0.0,
            ),
        },
        ButtonVariant::Ghost => match status {
            button::Status::Active => style(None, palette.text, Color::TRANSPARENT, 0.0),
            button::Status::Hovered => style(
                Some(darken(palette.background, 0.06)),
                palette.text,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Pressed => style(
                Some(darken(palette.background, 0.1)),
                darken(palette.text, 0.05),
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Disabled => {
                style(None, palette.text.scale_alpha(0.5), Color::TRANSPARENT, 0.0)
            }
        },
        ButtonVariant::Destructive => match status {
            button::Status::Active => style(
                Some(palette.danger.scale_alpha(0.12)),
                palette.danger,
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Hovered => style(
                Some(palette.danger.scale_alpha(0.2)),
                lighten(palette.danger, 0.06),
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Pressed => style(
                Some(palette.danger.scale_alpha(0.28)),
                darken(palette.danger, 0.04),
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Disabled => style(
                Some(palette.danger.scale_alpha(0.08)),
                palette.danger.scale_alpha(0.45),
                Color::TRANSPARENT,
                0.0,
            ),
        },
        ButtonVariant::Link => match status {
            button::Status::Active => style(None, palette.primary, Color::TRANSPARENT, 0.0),
            button::Status::Hovered => style(
                None,
                lighten(palette.primary, 0.08),
                Color::TRANSPARENT,
                0.0,
            ),
            button::Status::Pressed => {
                style(None, darken(palette.primary, 0.05), Color::TRANSPARENT, 0.0)
            }
            button::Status::Disabled => style(
                None,
                palette.primary.scale_alpha(0.45),
                Color::TRANSPARENT,
                0.0,
            ),
        },
    }
}

fn style(
    background: Option<Color>,
    text_color: Color,
    border_color: Color,
    border_width: f32,
) -> button::Style {
    button::Style {
        background: background.map(Background::Color),
        text_color,
        border: Border {
            color: border_color,
            width: border_width,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
