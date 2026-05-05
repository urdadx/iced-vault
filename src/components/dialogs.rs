use iced::widget::{button, column, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow, Theme};
use iced_dialog::dialog;

use crate::Message;
use crate::components::ui::{
    ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, button_style, darken, input,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogKind {
    NewVault,
    NewLogin,
    EditLogin,
    NewCard,
    EditCard,
    ImportComplete,
    ChangeMasterPassword,
}

pub(crate) struct DialogForms<'a> {
    pub(crate) new_vault_name: &'a str,
    pub(crate) login_website: &'a str,
    pub(crate) login_username: &'a str,
    pub(crate) login_password: &'a str,
    pub(crate) cardholder_name: &'a str,
    pub(crate) card_number: &'a str,
    pub(crate) expiration_date: &'a str,
    pub(crate) security_code: &'a str,
    pub(crate) import_result_message: &'a str,
    pub(crate) loading_message: Option<&'a str>,
    pub(crate) old_master_password: &'a str,
    pub(crate) new_master_password: &'a str,
    pub(crate) confirm_master_password: &'a str,
}

pub(crate) fn wrap<'a>(
    base: impl Into<Element<'a, Message>>,
    active_dialog: Option<DialogKind>,
    forms: DialogForms<'a>,
) -> Element<'a, Message> {
    let base = base.into();

    let Some(dialog_kind) = active_dialog else {
        return base;
    };

    let (dialog_title, dialog_content): (&'static str, Element<'a, Message>) = match dialog_kind {
        DialogKind::NewVault => (
            "New Vault",
            column![
                text("Vault name").size(14),
                input("Vault name", forms.new_vault_name).on_input(Message::AddVaultNameChanged),
                submit_button(
                    !forms.new_vault_name.trim().is_empty(),
                    Message::CreateVaultConfirmed,
                    forms.loading_message,
                )
            ]
            .spacing(12)
            .width(Length::Fill)
            .into(),
        ),
        DialogKind::NewLogin | DialogKind::EditLogin => (
            if dialog_kind == DialogKind::EditLogin {
                "Edit Login"
            } else {
                "New Login"
            },
            column![
                text("Website").size(14),
                input("https://example.com", forms.login_website)
                    .on_input(Message::LoginWebsiteChanged),
                text("Email / Username").size(14),
                input("Email / Username", forms.login_username)
                    .on_input(Message::LoginUsernameChanged),
                text("Password").size(14),
                input("Password", forms.login_password)
                    .secure(true)
                    .on_input(Message::LoginPasswordChanged),
                submit_button(
                    !forms.login_website.trim().is_empty()
                        && !forms.login_username.trim().is_empty()
                        && !forms.login_password.is_empty(),
                    if dialog_kind == DialogKind::EditLogin {
                        Message::UpdateLoginConfirmed
                    } else {
                        Message::CreateLoginConfirmed
                    },
                    forms.loading_message,
                )
            ]
            .spacing(12)
            .width(Length::Fill)
            .into(),
        ),
        DialogKind::NewCard | DialogKind::EditCard => (
            if dialog_kind == DialogKind::EditCard {
                "Edit Card"
            } else {
                "New Card"
            },
            column![
                text("Cardholder name").size(14),
                input("Cardholder name", forms.cardholder_name)
                    .on_input(Message::CardholderNameChanged),
                text("Number").size(14),
                input("Number", forms.card_number).on_input(Message::CardNumberChanged),
                text("Expiration date").size(14),
                input("Expiration date", forms.expiration_date)
                    .on_input(Message::ExpirationDateChanged),
                text("Security code").size(14),
                input("Security code", forms.security_code)
                    .secure(true)
                    .on_input(Message::SecurityCodeChanged),
                submit_button(
                    !forms.cardholder_name.trim().is_empty()
                        && !forms.card_number.trim().is_empty()
                        && !forms.expiration_date.trim().is_empty()
                        && !forms.security_code.trim().is_empty(),
                    if dialog_kind == DialogKind::EditCard {
                        Message::UpdateCardConfirmed
                    } else {
                        Message::CreateCardConfirmed
                    },
                    forms.loading_message,
                )
            ]
            .spacing(12)
            .width(Length::Fill)
            .into(),
        ),
        DialogKind::ChangeMasterPassword => (
            "Change Master Password",
            column![
                text("Current master password").size(14),
                input("Current master password", forms.old_master_password)
                    .secure(true)
                    .on_input(Message::OldMasterPasswordChanged),
                text("New master password").size(14),
                input("New master password", forms.new_master_password)
                    .secure(true)
                    .on_input(Message::NewMasterPasswordChanged),
                text("Confirm new password").size(14),
                input("Confirm new password", forms.confirm_master_password)
                    .secure(true)
                    .on_input(Message::ConfirmMasterPasswordChanged),
                submit_button(
                    !forms.old_master_password.is_empty()
                        && !forms.new_master_password.is_empty()
                        && forms.new_master_password == forms.confirm_master_password,
                    Message::ChangeMasterPasswordConfirmed,
                    forms.loading_message,
                )
            ]
            .spacing(12)
            .width(Length::Fill)
            .into(),
        ),
        DialogKind::ImportComplete => (
            "Import completed",
            column![
                text(forms.import_result_message).size(15),
                close_button("OK"),
            ]
            .spacing(16)
            .width(Length::Fill)
            .into(),
        ),
    };

    dialog(true, base, dialog_content)
        .title(dialog_title)
        .width(360)
        .padding_inner(18)
        .padding_outer(28)
        .spacing(14)
        .backdrop(Color::from_rgba(0.0, 0.0, 0.0, 0.45))
        .container_style(dialog_container_style)
        .title_style(dialog_title_style)
        .on_press(Message::CloseDialog)
        .into()
}

fn close_button<'a>(label: &'a str) -> Element<'a, Message> {
    button(label)
        .padding(ButtonSize::Sm.padding())
        .style(button_style(ButtonVariant::Default))
        .on_press(Message::CloseDialog)
        .into()
}

fn submit_button<'a>(
    enabled: bool,
    message: Message,
    loading_message: Option<&'a str>,
) -> Element<'a, Message> {
    let button = button(loading_message.unwrap_or("Submit"))
        .padding(ButtonSize::Sm.padding())
        .style(button_style(ButtonVariant::Default));

    if loading_message.is_some() {
        button.into()
    } else if enabled {
        button.on_press(message).into()
    } else {
        button.into()
    }
}

fn dialog_container_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        text_color: Some(palette.text),
        background: Some(Background::Color(darken(palette.background, 0.01))),
        border: Border {
            color: darken(palette.background, 0.16),
            width: 1.0,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn dialog_title_style(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().text),
    }
}
