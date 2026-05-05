use iced::widget::{button, column, container, text};
use iced::{Background, Border, Element, Length, Shadow, Theme};

use crate::Message;
use crate::components::ui::{
    ButtonSize, ButtonVariant, THEME_CORNER_RADIUS, button_style, darken, input,
};

pub(crate) fn view<'a>(
    needs_setup: bool,
    master_password: &'a str,
    master_password_confirmation: &'a str,
    status_message: Option<&'a str>,
    loading_message: Option<&'a str>,
) -> Element<'a, Message> {
    let title = if needs_setup {
        "Create Master Password"
    } else {
        "Unlock Your Vault"
    };
    let description = if needs_setup {
        "This password encrypts your local vault database. It is never stored, so it cannot be recovered."
    } else {
        "Enter your master password before accessing your vaults."
    };
    let button_label = if needs_setup {
        "Create encrypted vault"
    } else {
        "Unlock vault"
    };

    let mut fields = column![
        text(title).size(22),
        text(description).size(13),
        input("Master password", master_password)
            .secure(true)
            .on_input(Message::MasterPasswordChanged),
    ]
    .spacing(14)
    .width(Length::Fill);

    if needs_setup {
        fields = fields.push(
            input("Confirm master password", master_password_confirmation)
                .secure(true)
                .on_input(Message::MasterPasswordConfirmationChanged),
        );
    }

    let submit_label = loading_message.unwrap_or(button_label);
    let submit_button = button(text(submit_label).center().width(Length::Fill))
        .padding(ButtonSize::Sm.padding())
        .width(Length::Fill)
        .style(button_style(ButtonVariant::Default));

    fields = fields.push(if loading_message.is_some() {
        submit_button
    } else {
        submit_button.on_press(Message::UnlockConfirmed)
    });

    if let Some(message) = status_message {
        fields = fields.push(text(message).size(13).style(error_text_style));
    }

    container(container(fields).padding(24).width(420).style(panel_style))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}

fn error_text_style(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().danger),
    }
}

fn panel_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        text_color: Some(palette.text),
        background: Some(Background::Color(darken(palette.background, 0.02))),
        border: Border {
            color: darken(palette.background, 0.16),
            width: 1.0,
            radius: THEME_CORNER_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
