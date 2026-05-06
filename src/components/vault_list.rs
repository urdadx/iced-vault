use iced::widget::{Space, button, column, container, image, row, scrollable, svg, text};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme};

use crate::Message;
use crate::assets::SvgAsset;
use crate::components::ui::{THEME_CORNER_RADIUS, darken};
use crate::models::{ItemKind, VaultItem};

pub(crate) fn view<'a>(
    items: &'a [VaultItem],
    search_query: &'a str,
    browse_sort: Option<&'static str>,
) -> Element<'a, Message> {
    let search_query = search_query.trim().to_lowercase();
    let mut visible_items = items
        .iter()
        .filter(|item| {
            search_query.is_empty()
                || item.title.to_lowercase().contains(&search_query)
                || item.subtitle.to_lowercase().contains(&search_query)
                || item
                    .website_url
                    .as_ref()
                    .is_some_and(|url| url.to_lowercase().contains(&search_query))
        })
        .collect::<Vec<_>>();

    sort_items(&mut visible_items, browse_sort);

    let visible_count = visible_items.len();
    let recent_items = visible_items
        .into_iter()
        .fold(column![].spacing(10).width(Length::Fill), |items, item| {
            items.push(recent_item_card(item))
        });

    if visible_count == 0 {
        return empty_state();
    }

    scrollable(
        container(recent_items)
            .padding([0.0, 8.0])
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            Space::new().height(Length::Fill),
            svg(SvgAsset::Browse.handle()).width(96).height(96),
            text("No items added yet.").size(16),
            Space::new().height(Length::Fill),
        ]
        .spacing(16)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sort_items(items: &mut [&VaultItem], browse_sort: Option<&'static str>) {
    match browse_sort.unwrap_or("Recent") {
        "Oldest" => items.sort_by_key(|item| item.created_at),
        "A-Z" => items.sort_by(|left, right| sort_title(left).cmp(&sort_title(right))),
        "Z-A" => items.sort_by(|left, right| sort_title(right).cmp(&sort_title(left))),
        _ => items.sort_by_key(|item| std::cmp::Reverse(item.updated_at)),
    }
}

fn sort_title(item: &VaultItem) -> String {
    item.title.to_lowercase()
}

fn recent_item_card<'a>(item: &'a VaultItem) -> iced::widget::Button<'a, Message> {
    let title = truncate_link(&strip_url_scheme(&item.title), 42);
    let subtitle = truncate_link(&strip_url_scheme(&item.subtitle), 52);

    button(
        row![
            favicon_badge(item),
            column![
                text(title).size(14).width(Length::Fill),
                text(subtitle).size(13).width(Length::Fill),
            ]
            .spacing(2)
            .width(Length::Fill),
            svg(SvgAsset::ChevronRight.handle())
                .width(16)
                .height(16)
                .style(card_icon_style)
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding([12.0, 14.0])
    .width(Length::Fill)
    .style(item_card_style)
    .on_press(Message::OpenItem(item.id.clone()))
}

fn truncate_link(value: &str, max_chars: usize) -> String {
    if !is_link_like(value) || value.chars().count() <= max_chars {
        return value.to_owned();
    }

    value
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>()
        + "..."
}

fn is_link_like(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("www.")
        || value.contains('/')
}

fn strip_url_scheme(url: &str) -> String {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .to_owned()
}

fn favicon_badge<'a>(item: &'a VaultItem) -> Element<'a, Message> {
    if let Some(path) = &item.favicon_path {
        return container(image(path.as_str()).width(24).height(24))
            .padding([5.0, 6.0])
            .style(favicon_style)
            .into();
    }

    if item.kind == ItemKind::Card {
        return container(
            svg(SvgAsset::Card.handle())
                .width(24)
                .height(24)
                .style(card_icon_style),
        )
        .padding([5.0, 6.0])
        .style(favicon_style)
        .into();
    }

    if item.website_url.is_some() {
        return container(
            svg(SvgAsset::Globe.handle())
                .width(24)
                .height(24)
                .style(card_icon_style),
        )
        .padding([5.0, 6.0])
        .style(favicon_style)
        .into();
    }

    let badge = item.kind.label().chars().take(2).collect::<String>();

    container(text(badge).size(12))
        .padding([8.0, 10.0])
        .style(favicon_style)
        .into()
}

fn item_card_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let border = darken(palette.background, 0.15);

    let (background, border_color) = match status {
        button::Status::Active => (darken(palette.background, 0.02), border),
        button::Status::Hovered => (darken(palette.background, 0.05), darken(border, 0.05)),
        button::Status::Pressed => (darken(palette.background, 0.08), darken(border, 0.08)),
        button::Status::Disabled => (darken(palette.background, 0.01), border.scale_alpha(0.5)),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn favicon_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        text_color: Some(palette.primary),
        background: Some(Background::Color(palette.primary.scale_alpha(0.14))),
        border: Border {
            color: palette.primary.scale_alpha(0.4),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

fn card_icon_style(theme: &Theme, _status: iced::widget::svg::Status) -> iced::widget::svg::Style {
    iced::widget::svg::Style {
        color: Some(theme.palette().text.scale_alpha(0.6)),
    }
}
