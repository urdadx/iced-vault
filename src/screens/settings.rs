use iced::widget::{
    Space, button, column, container, pick_list, radio, row, scrollable, svg, text, text_input,
};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme};

use crate::Message;
use crate::components::ui::{ButtonVariant, THEME_CORNER_RADIUS, VaultTheme, button_style, darken};

// ---------------------------------------------------------------------------
// Sub-state types (add these fields to your top-level App / Model struct)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoLockDuration {
    OneMinute,
    FiveMinutes,
    #[default]
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    Never,
}

impl AutoLockDuration {
    pub const ALL: &'static [AutoLockDuration] = &[
        AutoLockDuration::OneMinute,
        AutoLockDuration::FiveMinutes,
        AutoLockDuration::TenMinutes,
        AutoLockDuration::ThirtyMinutes,
        AutoLockDuration::OneHour,
        AutoLockDuration::Never,
    ];
}

impl std::fmt::Display for AutoLockDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoLockDuration::OneMinute => write!(f, "1 minute"),
            AutoLockDuration::FiveMinutes => write!(f, "5 minutes"),
            AutoLockDuration::TenMinutes => write!(f, "10 minutes"),
            AutoLockDuration::ThirtyMinutes => write!(f, "30 minutes"),
            AutoLockDuration::OneHour => write!(f, "1 hour"),
            AutoLockDuration::Never => write!(f, "Never"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    PgpEncrypted,
    Zip,
    Csv,
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

/// Extra state needed by this view — thread it through from your Model.
pub struct SettingsViewState<'a> {
    pub current_theme: VaultTheme,
    pub auto_lock: AutoLockDuration,
    pub export_format: ExportFormat,
    /// Passphrase field value (only relevant when format == PgpEncrypted)
    pub export_passphrase: &'a str,
    pub show_passphrase: bool,
    pub status_message: Option<&'a str>,
}

pub(crate) fn view<'a>(state: SettingsViewState<'a>) -> Element<'a, Message> {
    let SettingsViewState {
        current_theme,
        auto_lock,
        export_format,
        export_passphrase,
        show_passphrase,
        status_message,
    } = state;

    scrollable(
        column![
            Space::new().height(42),
            row![
                Space::new().width(48),
                column![
                    text("Settings").size(22),
                    settings_status(status_message),
                    Space::new().height(24),
                    // ── Appearance ──────────────────────────────────────────────
                    settings_section(
                        "Appearance",
                        "Customize how the application looks and feels.",
                        column![
                            row![
                                text("Application Theme").size(14).width(Length::Fill),
                                theme_toggle(current_theme),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                        ]
                        .spacing(12),
                    ),
                    Space::new().height(24),
                    // ── Security ─────────────────────────────────────────────────
                    settings_section(
                        "Security",
                        "Manage your master password and security preferences.",
                        column![
                            // Auto-lock row
                            row![
                                text("Auto-lock after").size(14).width(Length::Fill),
                                pick_list(
                                    AutoLockDuration::ALL,
                                    Some(auto_lock),
                                    Message::AutoLockChanged,
                                )
                                .width(160)
                                .padding([8, 12])
                                .style(pick_list_style),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                            // Divider
                            horizontal_rule(),
                            // Change master password row
                            row![
                                text("Master Password").size(14).width(Length::Fill),
                                button(text("Change Master Password").size(12))
                                    .padding([8, 16])
                                    .style(button_style(ButtonVariant::Default))
                                    .on_press(Message::ChangeMasterPasswordPressed),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                        ]
                        .spacing(12),
                    ),
                    Space::new().height(24),
                    // ── Data Management ───────────────────────────────────────────
                    settings_section(
                        "Data Management",
                        "Export your vault data or manage your local database.",
                        export_section(export_format, export_passphrase, show_passphrase),
                    ),
                    Space::new().height(24),
                    // ── About ─────────────────────────────────────────────────────
                    settings_section(
                        "About",
                        "Information about Iced Vault.",
                        column![
                            row![
                                text("Version").size(14).width(Length::Fill),
                                text("0.1.0").size(14),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                        ]
                        .spacing(12),
                    ),
                ]
                .width(500),
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill),
            Space::new().height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Export sub-section
// ---------------------------------------------------------------------------

fn export_section<'a>(
    format: ExportFormat,
    passphrase: &'a str,
    show_passphrase: bool,
) -> impl Into<Element<'a, Message>> {
    let pgp_selected = format == ExportFormat::PgpEncrypted;
    let needs_passphrase = pgp_selected;
    let has_passphrase = !passphrase.trim().is_empty();
    let can_export = !needs_passphrase || has_passphrase;
    let export_label = match format {
        ExportFormat::PgpEncrypted => "Export PGP",
        ExportFormat::Zip => "Export ZIP",
        ExportFormat::Csv => "Export CSV",
    };

    let passphrase_group = if needs_passphrase {
        column![
            horizontal_rule(),
            text("Passphrase").size(14),
            text("The exported file will be encrypted using PGP and requires a strong passphrase.")
                .size(12)
                .color([0.5, 0.5, 0.5]),
            row![
                text_input("Enter passphrase", passphrase)
                    .on_input(Message::ExportPassphraseChanged)
                    .padding([8, 12])
                    .width(Length::Fill)
                    .style(text_input_style),
                Space::new().width(8),
                button(
                    svg(if show_passphrase {
                        "src/icons/eye_off_icon.svg"
                    } else {
                        "src/icons/eye_icon.svg"
                    })
                    .width(16)
                    .height(16),
                )
                .padding([8, 10])
                .style(button_style(ButtonVariant::Default))
                .on_press(Message::ToggleShowPassphrase),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(8)
    } else {
        column![]
    };

    column![
        // File format label
        text("File format").size(14),
        // Radio buttons
        radio(
            "PGP-encrypted (recommended)",
            ExportFormat::PgpEncrypted,
            Some(format),
            Message::ExportFormatChanged,
        )
        .size(14)
        .spacing(8),
        radio(
            "ZIP",
            ExportFormat::Zip,
            Some(format),
            Message::ExportFormatChanged,
        )
        .size(14)
        .spacing(8),
        radio(
            "CSV",
            ExportFormat::Csv,
            Some(format),
            Message::ExportFormatChanged,
        )
        .size(14)
        .spacing(8),
        // Conditional passphrase input
        passphrase_group,
        horizontal_rule(),
        // Export button — full width, disabled when passphrase required but empty
        button(
            container(text(export_label).size(13))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding([10, 16])
        .style(button_style(if can_export {
            ButtonVariant::Default
        } else {
            ButtonVariant::Secondary
        }))
        .on_press(Message::ExportVault),
    ]
    .spacing(10)
    .width(Length::Fill)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn settings_section<'a>(
    title: &'static str,
    description: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(title).size(18),
        text(description).size(13).color([0.5, 0.5, 0.5]),
        Space::new().height(12),
        container(content)
            .padding(20)
            .style(section_container_style)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

fn theme_toggle<'a>(current_theme: VaultTheme) -> Element<'a, Message> {
    let (label, icon_path) = if current_theme == VaultTheme::VaultDark {
        ("Dark Mode", "src/icons/dark_mode.svg")
    } else {
        ("Light Mode", "src/icons/light_mode.svg")
    };

    button(
        row![
            svg(icon_path).width(16).height(16).style(theme_icon_style),
            text(label).size(12)
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([8, 16])
    .on_press(Message::ThemePressed)
    .style(button_style(ButtonVariant::Default))
    .into()
}

fn settings_status<'a>(status_message: Option<&'a str>) -> Element<'a, Message> {
    if let Some(message) = status_message {
        text(message).size(12).color([0.5, 0.5, 0.5]).into()
    } else {
        Space::new().height(0).into()
    }
}

fn theme_icon_style(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(theme.palette().text.scale_alpha(0.7)),
    }
}

/// A thin horizontal separator line.
fn horizontal_rule<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(1))
        .style(|theme: &Theme| {
            let palette = theme.palette();
            container::Style {
                background: Some(Background::Color(darken(palette.background, 0.1))),
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn section_container_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(Background::Color(darken(palette.background, 0.03))),
        border: Border {
            radius: THEME_CORNER_RADIUS.into(),
            width: 1.0,
            color: darken(palette.background, 0.1),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    let base_bg = darken(palette.background, 0.04);

    text_input::Style {
        background: Background::Color(base_bg),
        border: Border {
            radius: THEME_CORNER_RADIUS.into(),
            width: 1.0,
            color: match status {
                text_input::Status::Focused { .. } => palette.primary,
                _ => darken(palette.background, 0.12),
            },
        },
        icon: palette.text.scale_alpha(0.5),
        placeholder: palette.text.scale_alpha(0.4),
        value: palette.text,
        selection: palette.primary.scale_alpha(0.3),
    }
}

fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();
    let base_bg = darken(palette.background, 0.06);

    pick_list::Style {
        background: Background::Color(base_bg),
        border: Border {
            radius: THEME_CORNER_RADIUS.into(),
            width: 1.0,
            color: match status {
                pick_list::Status::Hovered => darken(base_bg, 0.12),
                _ => darken(base_bg, 0.08),
            },
        },
        text_color: palette.text,
        placeholder_color: palette.text.scale_alpha(0.5),
        handle_color: palette.text.scale_alpha(0.6),
    }
}
