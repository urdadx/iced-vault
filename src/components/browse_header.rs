use iced::widget::{Space, column, pick_list, row, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::Message;
use crate::components::ui::{ButtonSize, THEME_CORNER_RADIUS, darken, input, s_pick_list};

const NEW_ITEM_TYPES: [&str; 2] = ["Login", "Card"];
const SORT_OPTIONS: [&str; 4] = ["Oldest", "Recent", "A-Z", "Z-A"];

pub(crate) fn view<'a>(
    search_query: &'a str,
    new_item_type: Option<&'static str>,
    browse_sort: Option<&'static str>,
) -> Element<'a, Message> {
    column![
        row![
            input("Browse vault items", search_query)
                .padding(ButtonSize::Sm.padding())
                .size(14)
                .on_input(Message::BrowseQueryChanged),
            pick_list(NEW_ITEM_TYPES, new_item_type, Message::NewItemTypeSelected)
                .placeholder("New item")
                .padding(ButtonSize::Sm.padding())
                .text_size(14)
                .width(120)
                .style(s_pick_list)
        ]
        .spacing(10)
        .width(Length::Fill),
        Space::new().height(18),
        row![
            text(browse_sort.unwrap_or("Recent")).size(16),
            Space::new().width(Length::Fill),
            pick_list(SORT_OPTIONS, browse_sort, Message::BrowseSortSelected)
                .padding(ButtonSize::Xs.padding())
                .text_size(13)
                .width(88)
                .style(borderless_pick_list_style)
        ]
        .width(Length::Fill)
        .align_y(iced::Alignment::Center)
    ]
    .into()
}

fn borderless_pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();
    let placeholder = palette.text.scale_alpha(0.55);

    match status {
        pick_list::Status::Active => pick_list::Style {
            text_color: palette.text.scale_alpha(0.8),
            placeholder_color: placeholder,
            handle_color: palette.text.scale_alpha(0.75),
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: THEME_CORNER_RADIUS.into(),
            },
        },
        pick_list::Status::Hovered => pick_list::Style {
            text_color: palette.text,
            placeholder_color: placeholder,
            handle_color: palette.text,
            background: Background::Color(darken(palette.background, 0.03)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: THEME_CORNER_RADIUS.into(),
            },
        },
        pick_list::Status::Opened { .. } => pick_list::Style {
            text_color: palette.text,
            placeholder_color: placeholder,
            handle_color: palette.primary,
            background: Background::Color(darken(palette.background, 0.03)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: THEME_CORNER_RADIUS.into(),
            },
        },
    }
}
