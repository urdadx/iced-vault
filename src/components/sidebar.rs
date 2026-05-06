use iced::widget::{Space, button, column, container, pick_list, row, svg, text};
use iced::{Background, Border, Color, Element, Length, Shadow, Theme};

use crate::assets::SvgAsset;
use crate::components::ui::{
    ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, VaultTheme, button_style, darken, s_pick_list,
};
use crate::{Message, Screen};

pub(crate) fn view<'a>(
    vaults: &'a [String],
    selected_vault: Option<&'a String>,
    active_screen: Screen,
    _theme: VaultTheme,
) -> Element<'a, Message> {
    column![
        container(
            row![
                pick_list(vaults, selected_vault, Message::VaultSelected)
                    .placeholder("Select vault")
                    .padding(ButtonSize::Sm.padding())
                    .text_size(14)
                    .width(Length::Fill)
                    .style(s_pick_list),
                button(
                    svg(SvgAsset::Plus.handle())
                        .width(16)
                        .height(16)
                        .style(theme_icon_style)
                )
                .padding(ButtonSize::Sm.padding())
                .width(34)
                .style(button_style(ButtonVariant::Default))
                .on_press(Message::AddVaultPressed)
            ]
            .spacing(8)
            .width(Length::Fill)
        )
        .width(Length::Fill),
        Space::new().height(40),
        column![
            nav_button(active_screen, Screen::Browse),
            nav_button(active_screen, Screen::Import),
            nav_button(active_screen, Screen::Settings),
        ]
        .spacing(20)
        .width(Length::Fill),
        Space::new().height(Length::Fill),
        container(
            button(
                row![
                    svg(SvgAsset::Github.handle())
                        .width(16)
                        .height(16)
                        .style(theme_icon_style),
                    text("Star on Github").size(14)
                ]
                .spacing(ButtonSize::Sm.gap())
            )
            .padding(ButtonSize::Sm.padding())
            .width(Length::Fill)
            .style(button_style(ButtonVariant::Ghost))
            .on_press(Message::OpenGithub)
        )
        .padding([10.0, 10.0])
        .width(Length::Fill)
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub(crate) fn divider_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette().text.scale_alpha(0.12))),
        ..Default::default()
    }
}

fn nav_button<'a>(active_screen: Screen, screen: Screen) -> iced::widget::Button<'a, Message> {
    let is_active = active_screen == screen;

    button(
        row![
            svg(screen.icon())
                .width(16)
                .height(16)
                .style(nav_icon_style(is_active)),
            text(screen.label()).size(14)
        ]
        .spacing(ButtonSize::Sm.gap()),
    )
    .padding(ButtonSize::Sm.padding())
    .width(Length::Fill)
    .style(nav_button_style(is_active))
    .on_press(Message::Navigate(screen))
}

fn nav_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme, status| {
        let palette = theme.palette();
        let pop_blue = Color::from_rgb8(0x48, 0xB9, 0xC7);

        if active {
            let background = match status {
                button::Status::Active => pop_blue,
                button::Status::Hovered => darken(pop_blue, 0.05),
                button::Status::Pressed => darken(pop_blue, 0.1),
                button::Status::Disabled => pop_blue.scale_alpha(0.35),
            };

            let text_color = match status {
                button::Status::Disabled => Color::from_rgba(1.0, 1.0, 1.0, 0.6),
                _ => Color::WHITE,
            };

            return button::Style {
                background: Some(Background::Color(background)),
                text_color,
                border: Border {
                    color: background,
                    width: 1.0,
                    radius: THEME_CORNER_RADIUS.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            };
        }

        let background = match status {
            button::Status::Active => Color::TRANSPARENT,
            button::Status::Hovered => darken(palette.background, 0.04),
            button::Status::Pressed => darken(palette.background, 0.08),
            button::Status::Disabled => Color::TRANSPARENT,
        };

        let text_color = match status {
            button::Status::Disabled => palette.text.scale_alpha(0.5),
            _ => palette.text,
        };

        button::Style {
            background: if background == Color::TRANSPARENT {
                None
            } else {
                Some(Background::Color(background))
            },
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: THEME_CORNER_RADIUS.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

fn nav_icon_style(
    active: bool,
) -> impl Fn(&Theme, iced::widget::svg::Status) -> iced::widget::svg::Style + Copy {
    move |theme, _status| iced::widget::svg::Style {
        color: Some(if active {
            Color::WHITE
        } else {
            theme.palette().text.scale_alpha(0.8)
        }),
    }
}

fn theme_icon_style(theme: &Theme, _status: iced::widget::svg::Status) -> iced::widget::svg::Style {
    iced::widget::svg::Style {
        color: Some(theme.palette().text.scale_alpha(0.8)),
    }
}
