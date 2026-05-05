mod browse;
mod import;
mod item_details;
mod master_password;
mod settings;

use iced::widget::{Space, column, text};
use iced::{Element, Length};

pub(crate) use browse::view as browse;
pub(crate) use import::view as import;
pub(crate) use item_details::view as item_details;
pub(crate) use master_password::view as master_password;
pub(crate) use settings::view as settings;
pub(crate) use settings::{AutoLockDuration, ExportFormat, SettingsViewState};

use crate::components::ui::VaultTheme;

pub(crate) fn placeholder<'a, Message: 'a>(
    title: &'static str,
    description: &'static str,
    empty_state: &'static str,
) -> Element<'a, Message> {
    column![
        text(title).size(18),
        text(description).size(13),
        Space::new().height(14),
        text(empty_state).size(14),
        Space::new().height(Length::Fill),
    ]
    .spacing(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
