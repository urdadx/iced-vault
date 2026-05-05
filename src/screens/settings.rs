use iced::Element;

use crate::Message;

pub(crate) fn view<'a>() -> Element<'a, Message> {
    super::placeholder(
        "Settings",
        "Adjust vault preferences and application settings.",
        "Settings controls will appear here.",
    )
}
