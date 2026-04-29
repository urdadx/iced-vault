mod components;

use iced::widget::{Space, button, column, container, pick_list, row, svg, text};
use iced::{Background, Border, Color, Element, Length, Shadow, Size, Theme};

use components::ui::{
    ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, VaultTheme, button_style, darken, load_fonts,
    p_font, s_pick_list,
};

const WINDOW_WIDTH: f32 = 820.0;
const WINDOW_HEIGHT: f32 = 570.0;
const SIDEBAR_WIDTH: f32 = 188.0;
const VAULTS: [&str; 3] = ["Vault 1", "Vault 2", "Vault 3"];

pub fn main() -> iced::Result {
    let mut app = iced::application(VaultApp::default, VaultApp::update, VaultApp::view)
        .window_size(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .theme(app_theme)
        .default_font(p_font())
        .centered();

    for font in load_fonts() {
        app = app.font(font);
    }

    app.run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Screen {
    #[default]
    Browse,
    Import,
    Settings,
}

impl Screen {
    const fn label(self) -> &'static str {
        match self {
            Self::Browse => "Browse",
            Self::Import => "Import",
            Self::Settings => "Settings",
        }
    }

    const fn icon_path(self) -> &'static str {
        match self {
            Self::Browse => "src/icons/browse_icon.svg",
            Self::Import => "src/icons/import_icon.svg",
            Self::Settings => "src/icons/settings_icon.svg",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Browse => "Browse the selected vault and inspect its contents.",
            Self::Import => "Import new records and files into the current vault.",
            Self::Settings => "Adjust vault preferences and application settings.",
        }
    }

    const fn empty_state(self) -> &'static str {
        match self {
            Self::Browse => "Vault items will appear here.",
            Self::Import => "Import actions and progress will appear here.",
            Self::Settings => "Settings controls will appear here.",
        }
    }
}

struct VaultApp {
    active_screen: Screen,
    selected_vault: Option<&'static str>,
    theme: VaultTheme,
}

impl Default for VaultApp {
    fn default() -> Self {
        Self {
            active_screen: Screen::Browse,
            selected_vault: Some(VAULTS[0]),
            theme: VaultTheme::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(Screen),
    VaultSelected(&'static str),
    AddVaultPressed,
    ThemePressed,
}

impl VaultApp {
    fn update(&mut self, message: Message) {
        match message {
            Message::Navigate(screen) => self.active_screen = screen,
            Message::VaultSelected(vault) => self.selected_vault = Some(vault),
            Message::AddVaultPressed => {}
            Message::ThemePressed => self.theme = self.theme.toggle(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content_panel = container(
            column![
                text(self.active_screen.label()).size(18),
                text(self.active_screen.description()).size(13),
                Space::new().height(14),
                text(self.active_screen.empty_state()).size(14),
                Space::new().height(Length::Fill),
            ]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .padding([16.0, 18.0])
        .width(Length::Fill)
        .height(Length::Fill);

        let sidebar = column![
            container(
                row![
                    pick_list(VAULTS, self.selected_vault, Message::VaultSelected)
                        .placeholder("Select vault")
                        .padding(ButtonSize::Sm.padding())
                        .text_size(14)
                        .width(Length::Fill)
                        .style(s_pick_list),
                    button(
                        svg("src/icons/plus_icon.svg")
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
                self.nav_button(Screen::Browse),
                self.nav_button(Screen::Import),
                self.nav_button(Screen::Settings),
            ]
            .spacing(20)
            .width(Length::Fill),
            Space::new().height(Length::Fill),
            container(
                button(
                    row![
                        svg(theme_icon_path(self.theme))
                            .width(16)
                            .height(16)
                            .style(theme_icon_style),
                        text(format!("Theme: {}", self.theme.label())).size(14)
                    ]
                    .spacing(ButtonSize::Sm.gap())
                )
                .padding(ButtonSize::Sm.padding())
                .width(Length::Fill)
                .style(button_style(ButtonVariant::Ghost))
                .on_press(Message::ThemePressed)
            )
            .padding([10.0, 10.0])
            .width(Length::Fill)
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            row![
                container(sidebar)
                    .width(SIDEBAR_WIDTH)
                    .height(Length::Fill)
                    .padding([0.0, 0.0]),
                content_panel,
            ]
            .spacing(12)
            .height(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn nav_button(&self, screen: Screen) -> iced::widget::Button<'_, Message> {
        let is_active = self.active_screen == screen;

        button(
            row![
                svg(screen.icon_path())
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
}

fn app_theme(app: &VaultApp) -> Theme {
    app.theme.to_iced_theme()
}

fn theme_icon_path(theme: VaultTheme) -> &'static str {
    match theme {
        VaultTheme::VaultLight => "src/icons/light_mode.svg",
        VaultTheme::VaultDark => "src/icons/dark_mode.svg",
    }
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
