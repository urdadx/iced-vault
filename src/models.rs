use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(crate) struct Vault {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VaultItem {
    pub(crate) id: String,
    pub(crate) kind: ItemKind,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
    pub(crate) website_url: Option<String>,
    pub(crate) favicon_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct VaultItemDetails {
    pub(crate) id: String,
    pub(crate) kind: ItemKind,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
    pub(crate) website_url: Option<String>,
    pub(crate) favicon_path: Option<String>,
    pub(crate) payload: ItemPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ItemKind {
    Login,
    Card,
}

impl ItemKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Card => "card",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Login => "Login",
            Self::Card => "Card",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        match value {
            "login" => Some(Self::Login),
            "card" => Some(Self::Card),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ItemPayload {
    Login(LoginPayload),
    Card(CardPayload),
}

impl ItemPayload {
    pub(crate) const fn kind(&self) -> ItemKind {
        match self {
            Self::Login(_) => ItemKind::Login,
            Self::Card(_) => ItemKind::Card,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self {
            Self::Login(login) => first_non_empty(&[&login.name, &login.username])
                .unwrap_or("Untitled login")
                .to_owned(),
            Self::Card(card) => first_non_empty(&[&card.cardholder_name, &card.brand])
                .unwrap_or("Untitled card")
                .to_owned(),
        }
    }

    pub(crate) fn subtitle(&self) -> String {
        match self {
            Self::Login(login) => first_non_empty(&[&login.username, &login.primary_url()])
                .unwrap_or(login.source.label())
                .to_owned(),
            Self::Card(card) => card
                .last_four()
                .map(|last_four| format!("Card ending in {last_four}"))
                .unwrap_or_else(|| card.source.label().to_owned()),
        }
    }

    pub(crate) fn website_url(&self) -> Option<String> {
        match self {
            Self::Login(login) => login.urls.first().cloned(),
            Self::Card(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoginPayload {
    pub(crate) name: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) urls: Vec<String>,
    pub(crate) notes: String,
    pub(crate) folder_path: String,
    pub(crate) custom_fields: Vec<CustomField>,
    pub(crate) source: ImportSource,
    pub(crate) source_item_id: Option<String>,
}

impl LoginPayload {
    pub(crate) fn manual(website: String, username: String, password: String) -> Self {
        Self {
            name: website.clone(),
            username,
            password,
            urls: vec![website],
            notes: String::new(),
            folder_path: String::new(),
            custom_fields: Vec::new(),
            source: ImportSource::Manual,
            source_item_id: None,
        }
    }

    fn primary_url(&self) -> String {
        self.urls.first().cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CardPayload {
    pub(crate) cardholder_name: String,
    pub(crate) number: String,
    pub(crate) expiration_date: String,
    pub(crate) security_code: String,
    pub(crate) brand: String,
    pub(crate) notes: String,
    pub(crate) folder_path: String,
    pub(crate) custom_fields: Vec<CustomField>,
    pub(crate) source: ImportSource,
    pub(crate) source_item_id: Option<String>,
}

impl CardPayload {
    pub(crate) fn manual(
        cardholder_name: String,
        number: String,
        expiration_date: String,
        security_code: String,
    ) -> Self {
        Self {
            cardholder_name,
            number,
            expiration_date,
            security_code,
            brand: String::new(),
            notes: String::new(),
            folder_path: String::new(),
            custom_fields: Vec::new(),
            source: ImportSource::Manual,
            source_item_id: None,
        }
    }

    fn last_four(&self) -> Option<String> {
        let digits: String = self
            .number
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect();

        (digits.len() >= 4).then(|| digits[digits.len() - 4..].to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CustomField {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportSource {
    Manual,
    Bitwarden,
    Chrome,
    Edge,
    Ipassword,
    Proton,
    Safari,
    Other(String),
}

impl ImportSource {
    pub(crate) fn label(&self) -> &str {
        match self {
            Self::Manual => "Manual",
            Self::Bitwarden => "Bitwarden",
            Self::Chrome => "Chrome",
            Self::Edge => "Edge",
            Self::Ipassword => "1Password",
            Self::Proton => "Proton",
            Self::Safari => "Safari",
            Self::Other(source) => source,
        }
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> Option<&'a str> {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
}
