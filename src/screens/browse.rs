use iced::widget::{Space, column};
use iced::{Element, Length};

use crate::Message;
use crate::components::{browse_header, vault_list};
use crate::models::{VaultItem, VaultItemDetails};

pub(crate) fn view<'a>(
    search_query: &'a str,
    new_item_type: Option<&'static str>,
    browse_sort: Option<&'static str>,
    vault_items: &'a [VaultItem],
    selected_item: Option<&'a VaultItemDetails>,
    show_password: bool,
) -> Element<'a, Message> {
    if let Some(item) = selected_item {
        return super::item_details(item, show_password);
    }

    column![
        browse_header::view(search_query, new_item_type, browse_sort),
        vault_list::view(vault_items, search_query, browse_sort),
        Space::new().height(Length::Shrink),
    ]
    .spacing(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
