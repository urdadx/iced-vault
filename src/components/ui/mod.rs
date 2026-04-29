mod button;
mod input;
mod picklist;

use iced::{Color, Font, Theme, color};

#[allow(unused_imports)]
pub use button::{ButtonSize, ButtonVariant, button_style};
#[allow(unused_imports)]
pub use input::{input, input_style};

#[allow(unused_imports)]
pub use picklist::s_pick_list;

pub const THEME_CORNER_RADIUS: f32 = 4.0;
pub const THEME_FONT_SIZE: f32 = 12.0;
pub const THEME_PADDING: f32 = 9.0;
pub const THEME_ICON_SIZE: f32 = 12.0;
pub const POP_BLUE: Color = color!(0x48b9c7);

#[allow(unused)]
pub fn load_fonts() -> Vec<std::borrow::Cow<'static, [u8]>> {
    vec![include_bytes!("./Inter-Regular.ttf").as_slice().into()]
}

pub fn p_font() -> Font {
    Font {
        family: iced::font::Family::Name("Inter".into()),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultTheme {
    VaultLight,
    VaultDark,
}

impl VaultTheme {
    pub fn to_iced_theme(self) -> Theme {
        match self {
            Self::VaultLight => Self::vault_light(),
            Self::VaultDark => Self::vault_dark(),
        }
    }

    pub const fn toggle(self) -> Self {
        match self {
            Self::VaultLight => Self::VaultDark,
            Self::VaultDark => Self::VaultLight,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::VaultLight => "light",
            Self::VaultDark => "dark",
        }
    }

    fn vault_light() -> Theme {
        Theme::custom(
            "Vault Light".to_string(),
            iced::theme::Palette {
                background: color!(0xf2ebe6),
                text: color!(0x3a3131),
                primary: POP_BLUE,
                success: color!(0xd9a6b3),
                danger: color!(0xe78a8a),
                warning: color!(0xf4b8c4),
            },
        )
    }

    fn vault_dark() -> Theme {
        Theme::custom(
            "Vault Dark".to_string(),
            iced::theme::Palette {
                background: color!(0x242424),
                text: color!(0xf2f2f2),
                primary: POP_BLUE,
                success: color!(0x4b5358),
                danger: color!(0xf28b82),
                warning: color!(0xe5c07b),
            },
        )
    }
}

impl Default for VaultTheme {
    fn default() -> Self {
        Self::VaultDark
    }
}

pub fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

pub fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}
