use iced::widget::{Space, button, column, image, row, text};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme};

use crate::Message;
use crate::assets::ImageAsset;
use crate::components::ui::{THEME_CORNER_RADIUS, darken};

const IMPORT_PROVIDERS: [ImportProvider; 6] = [
    ImportProvider {
        label: "Bitwarden",
        icon: ImageAsset::Bitwarden,
    },
    ImportProvider {
        label: "Chrome",
        icon: ImageAsset::Chrome,
    },
    ImportProvider {
        label: "Edge",
        icon: ImageAsset::Edge,
    },
    ImportProvider {
        label: "1Password",
        icon: ImageAsset::OnePassword,
    },
    ImportProvider {
        label: "Proton",
        icon: ImageAsset::Proton,
    },
    ImportProvider {
        label: "Safari",
        icon: ImageAsset::Safari,
    },
];

#[derive(Debug, Clone, Copy)]
struct ImportProvider {
    label: &'static str,
    icon: ImageAsset,
}

pub(crate) fn view<'a>(_status_message: Option<&'a str>) -> Element<'a, Message> {
    let first_row = row![
        import_card(IMPORT_PROVIDERS[0]),
        import_card(IMPORT_PROVIDERS[1]),
        import_card(IMPORT_PROVIDERS[2]),
    ]
    .spacing(40)
    .align_y(Alignment::Center);

    let second_row = row![
        import_card(IMPORT_PROVIDERS[3]),
        import_card(IMPORT_PROVIDERS[4]),
        import_card(IMPORT_PROVIDERS[5]),
    ]
    .spacing(40);

    column![
        Space::new().height(42),
        row![
            Space::new().width(48),
            column![
                text("Import your passwords").size(22),
                text("Upload a .csv or .json file of your passwords").size(14),
                first_row,
                second_row
            ]
            .spacing(36)
            .width(Length::Shrink),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn import_card<'a>(provider: ImportProvider) -> iced::widget::Button<'a, Message> {
    button(
        column![
            image(provider.icon.handle()).width(48).height(48),
            text(provider.label).size(16),
        ]
        .spacing(14)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding([22.0, 12.0])
    .width(134)
    .height(128)
    .style(import_card_style)
    .on_press(Message::ImportProviderSelected(provider.label))
}

fn import_card_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let border = darken(palette.background, 0.18);

    let background = match status {
        button::Status::Active => darken(palette.background, 0.01),
        button::Status::Hovered => darken(palette.background, 0.05),
        button::Status::Pressed => darken(palette.background, 0.08),
        button::Status::Disabled => darken(palette.background, 0.01),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text,
        border: Border {
            color: border,
            width: 1.0,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
