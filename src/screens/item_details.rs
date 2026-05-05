use chrono::{DateTime, Local};
use iced::widget::{Space, button, column, container, image, row, scrollable, svg, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::Message;
use crate::components::ui::{ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, button_style};
use crate::models::{ItemPayload, VaultItemDetails};

pub(crate) fn view(item: &VaultItemDetails, show_password: bool) -> Element<'_, Message> {
    let title = truncate_text(&item.title, 42);

    scrollable(
        column![
            row![
                button(
                    row![
                        svg("src/icons/back_icon.svg")
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
                        svg("src/icons/trash_icon.svg")
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

    container(
        svg("src/icons/globe.svg")
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
                detail_row("src/icons/letter_icon.svg", "Email", login.username.clone()),
                password_row(show_password, &login.password),
            ]),
            section_card(column![website_row(
                "src/icons/globe.svg",
                truncate_text(&item.website_url.clone().unwrap_or_default(), 68),
            )]),
            section_card(column![
                detail_row(
                    "src/icons/zap_icon.svg",
                    "Created",
                    format_timestamp(item.created_at)
                ),
                detail_row(
                    "src/icons/pencil_icon.svg",
                    "Last modified",
                    format_timestamp(item.updated_at),
                ),
            ]),
        ]
        .spacing(10)
        .into(),
        ItemPayload::Card(card) => column![
            section_card(column![
                detail_row(
                    "src/icons/letter_icon.svg",
                    "Cardholder",
                    card.cardholder_name.clone()
                ),
                detail_row(
                    "src/icons/key_icon.svg",
                    "Number",
                    masked_card(&card.number)
                ),
                detail_row(
                    "src/icons/pencil_icon.svg",
                    "Expiration",
                    card.expiration_date.clone()
                ),
                detail_row("src/icons/key_icon.svg", "Security code", "•••".to_owned()),
            ]),
            section_card(column![
                detail_row(
                    "src/icons/zap_icon.svg",
                    "Created",
                    format_timestamp(item.created_at)
                ),
                detail_row(
                    "src/icons/pencil_icon.svg",
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

fn detail_row<'a>(
    icon_path: &'static str,
    label: &'static str,
    value: String,
) -> Element<'a, Message> {
    row![
        svg(icon_path).width(18).height(18).style(icon_style),
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

    row![
        svg("src/icons/key_icon.svg")
            .width(18)
            .height(18)
            .style(icon_style),
        column![text("Password").size(12), text(display_value).size(14)]
            .spacing(2)
            .width(Length::Fill),
        button(
            svg(if show_password {
                "src/icons/eye_off_icon.svg"
            } else {
                "src/icons/eye_icon.svg"
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

fn website_row<'a>(icon_path: &'static str, value: String) -> Element<'a, Message> {
    row![
        svg(icon_path).width(18).height(18).style(icon_style),
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
