use iced::widget::{image, svg};
use iced::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvgAsset {
    Back,
    Browse,
    Card,
    ChevronRight,
    DarkMode,
    Eye,
    EyeOff,
    Github,
    Globe,
    Import,
    Key,
    Letter,
    LightMode,
    Pencil,
    Plus,
    Settings,
    Trash,
    Zap,
}

impl SvgAsset {
    pub(crate) fn handle(self) -> svg::Handle {
        svg::Handle::from_memory(match self {
            Self::Back => include_bytes!("icons/back_icon.svg").as_slice(),
            Self::Browse => include_bytes!("icons/browse_icon.svg").as_slice(),
            Self::Card => include_bytes!("icons/card_icon.svg").as_slice(),
            Self::ChevronRight => include_bytes!("icons/chevron_right.svg").as_slice(),
            Self::DarkMode => include_bytes!("icons/dark_mode.svg").as_slice(),
            Self::Eye => include_bytes!("icons/eye_icon.svg").as_slice(),
            Self::EyeOff => include_bytes!("icons/eye_off_icon.svg").as_slice(),
            Self::Github => include_bytes!("icons/BiGithub.svg").as_slice(),
            Self::Globe => include_bytes!("icons/globe.svg").as_slice(),
            Self::Import => include_bytes!("icons/import_icon.svg").as_slice(),
            Self::Key => include_bytes!("icons/key_icon.svg").as_slice(),
            Self::Letter => include_bytes!("icons/letter_icon.svg").as_slice(),
            Self::LightMode => include_bytes!("icons/light_mode.svg").as_slice(),
            Self::Pencil => include_bytes!("icons/pencil_icon.svg").as_slice(),
            Self::Plus => include_bytes!("icons/plus_icon.svg").as_slice(),
            Self::Settings => include_bytes!("icons/settings_icon.svg").as_slice(),
            Self::Trash => include_bytes!("icons/trash_icon.svg").as_slice(),
            Self::Zap => include_bytes!("icons/zap_icon.svg").as_slice(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageAsset {
    OnePassword,
    Bitwarden,
    Chrome,
    Edge,
    Proton,
    Safari,
}

impl ImageAsset {
    pub(crate) fn handle(self) -> image::Handle {
        image::Handle::from_bytes(match self {
            Self::OnePassword => include_bytes!("icons/1Password_icon.png").as_slice(),
            Self::Bitwarden => include_bytes!("icons/icons8-bitwarden-48.png").as_slice(),
            Self::Chrome => include_bytes!("icons/icons8-chrome-48.png").as_slice(),
            Self::Edge => include_bytes!("icons/icons8-edge-48.png").as_slice(),
            Self::Proton => include_bytes!("icons/proton.png").as_slice(),
            Self::Safari => include_bytes!("icons/safari.png").as_slice(),
        })
    }
}

pub(crate) fn app_icon() -> Option<window::Icon> {
    window::icon::from_file_data(include_bytes!("icons/app_icon.png"), None).ok()
}
