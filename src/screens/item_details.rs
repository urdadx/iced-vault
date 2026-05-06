use chrono::{DateTime, Local};
use iced::widget::{Space, button, column, container, image, row, scrollable, svg, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::Message;
use crate::assets::SvgAsset;
use crate::components::ui::{ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, button_style};
use crate::models::{ItemKind, ItemPayload, VaultItemDetails};

pub(crate) fn view(item: &VaultItemDetails, show_password: bool) -> Element<'_, Message> {
    let title = truncate_text(&item.title, 42);

    scrollable(
        column![
            row![
                button(
                    row![
                        svg(SvgAsset::Back.handle())
                            .width(16)
                            .height(16)
                            .style(icon_style),
                        text("Back").size(14),
                    ]
                    .spacing(4),
                )
                .padding(ButtonSize::Xs.padding())
                .style(button_style(ButtonVariant::Ghost))
                .on_press(Message::BackToVaultList),
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill),
            row![
                header_icon(item),
                text(title).size(18),
                Space::new().width(Length::Fill),
                button("Edit")
                    .padding(ButtonSize::Sm.padding())
                    .style(button_style(ButtonVariant::Outline))
                    .on_press(Message::EditSelectedItem),
                button(
                    row![
                        svg(SvgAsset::Trash.handle())
                            .width(16)
                            .height(16)
                            .style(icon_style),
                    ]
                    .spacing(4),
                )
                .padding(ButtonSize::Sm.padding())
                .style(button_style(ButtonVariant::Destructive))
                .on_press(Message::DeleteSelectedItem),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill),
            details_body(item, show_password),
            Space::new().height(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn header_icon<'a>(item: &'a VaultItemDetails) -> Element<'a, Message> {
    if let Some(path) = &item.favicon_path {
        return container(image(path.as_str()).width(32).height(32))
            .padding(8)
            .style(section_style)
            .into();
    }

    if item.kind == ItemKind::Card {
        return container(
            svg(SvgAsset::Card.handle())
                .width(20)
                .height(20)
                .style(icon_style),
        )
        .padding(8)
        .style(section_style)
        .into();
    }

    container(
        svg(SvgAsset::Globe.handle())
            .width(20)
            .height(20)
            .style(icon_style),
    )
    .padding(8)
    .style(section_style)
    .into()
}

fn details_body(item: &VaultItemDetails, show_password: bool) -> Element<'_, Message> {
    match &item.payload {
        ItemPayload::Login(login) => column![
            section_card(column![
                detail_row(SvgAsset::Letter, "Email", login.username.clone()),
                password_row(show_password, &login.password),
            ]),
            section_card(column![website_row(
                SvgAsset::Globe,
                truncate_text(&item.website_url.clone().unwrap_or_default(), 68),
            )]),
            section_card(column![
                detail_row(SvgAsset::Zap, "Created", format_timestamp(item.created_at)),
                detail_row(
                    SvgAsset::Pencil,
                    "Last modified",
                    format_timestamp(item.updated_at),
                ),
            ]),
        ]
        .spacing(10)
        .into(),
        ItemPayload::Card(card) => column![
            section_card(column![
                detail_row(SvgAsset::Letter, "Cardholder", card.cardholder_name.clone()),
                card_number_row(show_password, &card.number),
                detail_row(SvgAsset::Pencil, "Expiration", card.expiration_date.clone()),
                security_code_row(show_password, &card.security_code),
            ]),
            section_card(column![
                detail_row(SvgAsset::Zap, "Created", format_timestamp(item.created_at)),
                detail_row(
                    SvgAsset::Pencil,
                    "Last modified",
                    format_timestamp(item.updated_at),
                ),
            ]),
        ]
        .spacing(10)
        .into(),
    }
}

fn section_card<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content.spacing(0).width(Length::Fill))
        .padding([10.0, 12.0])
        .width(Length::Fill)
        .style(section_style)
        .into()
}

fn detail_row<'a>(icon: SvgAsset, label: &'static str, value: String) -> Element<'a, Message> {
    row![
        svg(icon.handle()).width(18).height(18).style(icon_style),
        column![
            text(label).size(12),
            text(value).size(14).width(Length::Fill)
        ]
        .spacing(2)
        .width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([8.0, 0.0])
    .width(Length::Fill)
    .into()
}

fn password_row<'a>(show_password: bool, password: &str) -> Element<'a, Message> {
    let display_value = if show_password {
        password.to_owned()
    } else {
        masked_password(password)
    };

    sensitive_detail_row(SvgAsset::Key, "Password", display_value, show_password)
}

fn card_number_row<'a>(show_number: bool, number: &str) -> Element<'a, Message> {
    let display_value = if show_number {
        number.to_owned()
    } else {
        masked_card(number)
    };

    sensitive_detail_row(SvgAsset::Key, "Number", display_value, show_number)
}

fn security_code_row<'a>(show_code: bool, code: &str) -> Element<'a, Message> {
    let display_value = if show_code {
        code.to_owned()
    } else {
        masked_security_code(code)
    };

    sensitive_detail_row(SvgAsset::Key, "Security code", display_value, show_code)
}

fn sensitive_detail_row<'a>(
    icon: SvgAsset,
    label: &'static str,
    value: String,
    is_visible: bool,
) -> Element<'a, Message> {
    row![
        svg(icon.handle()).width(18).height(18).style(icon_style),
        column![text(label).size(12), text(value).size(14)]
            .spacing(2)
            .width(Length::Fill),
        button(
            svg(if is_visible {
                SvgAsset::EyeOff.handle()
            } else {
                SvgAsset::Eye.handle()
            })
            .width(16)
            .height(16)
            .style(icon_style),
        )
        .padding(4)
        .style(button_style(ButtonVariant::Ghost))
        .on_press(Message::TogglePasswordVisibility),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([8.0, 0.0])
    .width(Length::Fill)
    .into()
}

fn website_row<'a>(icon: SvgAsset, value: String) -> Element<'a, Message> {
    row![
        svg(icon.handle()).width(18).height(18).style(icon_style),
        text(value).size(14).width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([8.0, 0.0])
    .width(Length::Fill)
    .into()
}

fn section_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let border_color = palette.primary.scale_alpha(0.28);

    container::Style {
        text_color: Some(palette.text),
        background: Some(Background::Color(palette.primary.scale_alpha(0.04))),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn masked_password(password: &str) -> String {
    "•".repeat(password.chars().count().clamp(8, 16))
}

fn masked_card(number: &str) -> String {
    let digits: String = number
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    if digits.len() < 4 {
        return String::from("••••");
    }

    format!("•••• •••• •••• {}", &digits[digits.len() - 4..])
}

fn masked_security_code(code: &str) -> String {
    let count = code.chars().count().clamp(3, 4);

    "•".repeat(count)
}

fn format_timestamp(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|date_time| {
            date_time
                .with_timezone(&Local)
                .format("%b %-d, %Y at %-I:%M %p")
                .to_string()
        })
        .unwrap_or_else(|| String::from("Unknown"))
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    value
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>()
        + "..."
}

fn icon_style(theme: &Theme, _status: iced::widget::svg::Status) -> iced::widget::svg::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r < 0.5;
    iced::widget::svg::Style {
        color: Some(if is_dark { Color::WHITE } else { palette.text }),
    }
}
