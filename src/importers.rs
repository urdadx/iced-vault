use std::collections::HashMap;

use serde::Deserialize;

use crate::models::{CardPayload, CustomField, ImportSource, ItemPayload, LoginPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImportFormat {
    BitwardenCsv,
    BitwardenJson,
    ChromeCsv,
    EdgeCsv,
    OnePasswordCsv,
    ProtonCsv,
    SafariCsv,
}

impl ImportFormat {
    pub(crate) const fn source(self) -> ImportSource {
        match self {
            Self::BitwardenCsv | Self::BitwardenJson => ImportSource::Bitwarden,
            Self::ChromeCsv => ImportSource::Chrome,
            Self::EdgeCsv => ImportSource::Edge,
            Self::OnePasswordCsv => ImportSource::Ipassword,
            Self::ProtonCsv => ImportSource::Proton,
            Self::SafariCsv => ImportSource::Safari,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::BitwardenCsv => "Bitwarden CSV",
            Self::BitwardenJson => "Bitwarden JSON",
            Self::ChromeCsv => "Chrome CSV",
            Self::EdgeCsv => "Edge CSV",
            Self::OnePasswordCsv => "1Password CSV",
            Self::ProtonCsv => "Proton CSV",
            Self::SafariCsv => "Safari CSV",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ImportPreview {
    pub(crate) format: ImportFormat,
    pub(crate) items: Vec<ItemPayload>,
    pub(crate) skipped: Vec<SkippedImportRow>,
}

impl ImportPreview {
    pub(crate) fn imported_count(&self) -> usize {
        self.items.len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SkippedImportRow {
    pub(crate) row_number: usize,
    pub(crate) reason: String,
}

#[derive(Debug)]
pub(crate) enum ImportError {
    Csv(csv::Error),
    Json(serde_json::Error),
    UnsupportedBitwardenJson,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv(error) => write!(formatter, "CSV import failed: {error}"),
            Self::Json(error) => write!(formatter, "JSON import failed: {error}"),
            Self::UnsupportedBitwardenJson => write!(
                formatter,
                "Bitwarden JSON import must be an unencrypted export with an items array",
            ),
        }
    }
}

impl std::error::Error for ImportError {}

impl From<csv::Error> for ImportError {
    fn from(error: csv::Error) -> Self {
        Self::Csv(error)
    }
}

impl From<serde_json::Error> for ImportError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub(crate) fn parse_import(
    format: ImportFormat,
    contents: &str,
) -> Result<ImportPreview, ImportError> {
    let (items, skipped) = match format {
        ImportFormat::BitwardenCsv => parse_bitwarden_csv(contents)?,
        ImportFormat::BitwardenJson => parse_bitwarden_json(contents)?,
        ImportFormat::ChromeCsv => parse_browser_csv(contents, ImportSource::Chrome)?,
        ImportFormat::EdgeCsv => parse_browser_csv(contents, ImportSource::Edge)?,
        ImportFormat::OnePasswordCsv => parse_one_password_csv(contents)?,
        ImportFormat::ProtonCsv => parse_proton_csv(contents)?,
        ImportFormat::SafariCsv => parse_browser_csv(contents, ImportSource::Safari)?,
    };

    Ok(ImportPreview {
        format,
        items,
        skipped,
    })
}

fn parse_bitwarden_csv(
    contents: &str,
) -> Result<(Vec<ItemPayload>, Vec<SkippedImportRow>), ImportError> {
    let mut items = Vec::new();
    let mut skipped = Vec::new();
    let mut reader = csv_reader(contents);
    let headers = HeaderMap::new(reader.headers()?);

    for (index, row) in reader.records().enumerate() {
        let record = row?;
        let row_number = index + 2;
        let item_type = headers.get(&record, &["type"]);

        match item_type.as_deref() {
            Some("login") => items.push(ItemPayload::Login(LoginPayload {
                name: headers.get(&record, &["name"]).unwrap_or_default(),
                username: headers
                    .get(&record, &["login_username", "username"])
                    .unwrap_or_default(),
                password: headers
                    .get(&record, &["login_password", "password"])
                    .unwrap_or_default(),
                urls: collect_values(&[headers.get(&record, &["login_uri", "url", "uri"])]),
                notes: headers.get(&record, &["notes"]).unwrap_or_default(),
                folder_path: headers
                    .get(&record, &["folder", "folder_path"])
                    .unwrap_or_default(),
                custom_fields: parse_bitwarden_custom_fields(
                    headers.get(&record, &["fields"]).as_deref(),
                ),
                source: ImportSource::Bitwarden,
                source_item_id: headers.get(&record, &["id"]),
            })),
            Some("card") => items.push(ItemPayload::Card(CardPayload {
                cardholder_name: headers
                    .get(&record, &["cardholder_name", "cardholdername"])
                    .unwrap_or_default(),
                number: headers
                    .get(&record, &["number", "card_number"])
                    .unwrap_or_default(),
                expiration_date: join_expiration(
                    headers.get(&record, &["exp_month", "expiration_month"]),
                    headers.get(&record, &["exp_year", "expiration_year"]),
                ),
                security_code: headers
                    .get(&record, &["code", "security_code"])
                    .unwrap_or_default(),
                brand: headers.get(&record, &["brand"]).unwrap_or_default(),
                notes: headers.get(&record, &["notes"]).unwrap_or_default(),
                folder_path: headers
                    .get(&record, &["folder", "folder_path"])
                    .unwrap_or_default(),
                custom_fields: parse_bitwarden_custom_fields(
                    headers.get(&record, &["fields"]).as_deref(),
                ),
                source: ImportSource::Bitwarden,
                source_item_id: headers.get(&record, &["id"]),
            })),
            Some(other) => skipped.push(SkippedImportRow {
                row_number,
                reason: format!("unsupported Bitwarden item type `{other}`"),
            }),
            None => skipped.push(SkippedImportRow {
                row_number,
                reason: String::from("missing Bitwarden item type"),
            }),
        }
    }

    Ok((items, skipped))
}

fn parse_browser_csv(
    contents: &str,
    source: ImportSource,
) -> Result<(Vec<ItemPayload>, Vec<SkippedImportRow>), ImportError> {
    let mut items = Vec::new();
    let mut skipped = Vec::new();
    let mut reader = csv_reader(contents);
    let headers = HeaderMap::new(reader.headers()?);

    for (index, row) in reader.records().enumerate() {
        let record = row?;
        let row_number = index + 2;
        let username = headers
            .get(&record, &["username", "user name", "login_username"])
            .unwrap_or_default();
        let password = headers.get(&record, &["password"]).unwrap_or_default();

        if username.is_empty() && password.is_empty() {
            skipped.push(SkippedImportRow {
                row_number,
                reason: String::from("missing username and password"),
            });
            continue;
        }

        items.push(ItemPayload::Login(LoginPayload {
            name: headers
                .get(&record, &["name", "title"])
                .or_else(|| headers.get(&record, &["url", "origin_url", "action_url"]))
                .unwrap_or_else(|| String::from("Imported login")),
            username,
            password,
            urls: collect_values(&[
                headers.get(&record, &["url", "origin_url"]),
                headers.get(&record, &["action_url"]),
            ]),
            notes: headers.get(&record, &["note", "notes"]).unwrap_or_default(),
            folder_path: String::new(),
            custom_fields: Vec::new(),
            source: source.clone(),
            source_item_id: None,
        }));
    }

    Ok((items, skipped))
}

fn parse_one_password_csv(
    contents: &str,
) -> Result<(Vec<ItemPayload>, Vec<SkippedImportRow>), ImportError> {
    let mut items = Vec::new();
    let mut skipped = Vec::new();
    let mut reader = csv_reader(contents);
    let headers = HeaderMap::new(reader.headers()?);

    for (index, row) in reader.records().enumerate() {
        let record = row?;
        let row_number = index + 2;
        let username = headers
            .get(&record, &["username", "user name", "login"])
            .unwrap_or_default();
        let password = headers.get(&record, &["password"]).unwrap_or_default();
        let url = headers
            .get(&record, &["website", "url", "urls", "login_uri"])
            .unwrap_or_default();

        if username.is_empty() && password.is_empty() && url.is_empty() {
            skipped.push(SkippedImportRow {
                row_number,
                reason: String::from("missing login fields"),
            });
            continue;
        }

        items.push(ItemPayload::Login(LoginPayload {
            name: headers
                .get(&record, &["title", "name"])
                .unwrap_or_else(|| String::from("Imported login")),
            username,
            password,
            urls: collect_values(&[Some(url)]),
            notes: headers.get(&record, &["notes", "note"]).unwrap_or_default(),
            folder_path: headers
                .get(&record, &["vault", "folder", "tags"])
                .unwrap_or_default(),
            custom_fields: Vec::new(),
            source: ImportSource::Ipassword,
            source_item_id: headers.get(&record, &["uuid", "id"]),
        }));
    }

    Ok((items, skipped))
}

fn parse_proton_csv(
    contents: &str,
) -> Result<(Vec<ItemPayload>, Vec<SkippedImportRow>), ImportError> {
    let mut items = Vec::new();
    let mut skipped = Vec::new();
    let mut reader = csv_reader(contents);
    let headers = HeaderMap::new(reader.headers()?);

    for (index, row) in reader.records().enumerate() {
        let record = row?;
        let row_number = index + 2;
        let username = headers
            .get(&record, &["username", "email", "login"])
            .unwrap_or_default();
        let password = headers.get(&record, &["password"]).unwrap_or_default();
        let url = headers
            .get(&record, &["url", "urls", "website", "websites"])
            .unwrap_or_default();

        if username.is_empty() && password.is_empty() && url.is_empty() {
            skipped.push(SkippedImportRow {
                row_number,
                reason: String::from("missing login fields"),
            });
            continue;
        }

        items.push(ItemPayload::Login(LoginPayload {
            name: headers
                .get(&record, &["name", "title"])
                .unwrap_or_else(|| String::from("Imported login")),
            username,
            password,
            urls: collect_values(&[Some(url)]),
            notes: headers.get(&record, &["note", "notes"]).unwrap_or_default(),
            folder_path: headers
                .get(&record, &["vault", "folder", "folder_path"])
                .unwrap_or_default(),
            custom_fields: Vec::new(),
            source: ImportSource::Proton,
            source_item_id: headers.get(&record, &["id", "item_id"]),
        }));
    }

    Ok((items, skipped))
}

fn parse_bitwarden_json(
    contents: &str,
) -> Result<(Vec<ItemPayload>, Vec<SkippedImportRow>), ImportError> {
    let export: BitwardenJsonExport = serde_json::from_str(contents)?;
    let Some(json_items) = export.items else {
        return Err(ImportError::UnsupportedBitwardenJson);
    };

    let mut items = Vec::new();
    let mut skipped = Vec::new();

    for (index, item) in json_items.into_iter().enumerate() {
        let row_number = index + 1;

        match item.item_type {
            1 => {
                let Some(login) = item.login else {
                    skipped.push(SkippedImportRow {
                        row_number,
                        reason: String::from("Bitwarden login item is missing login data"),
                    });
                    continue;
                };

                items.push(ItemPayload::Login(LoginPayload {
                    name: item.name.unwrap_or_default(),
                    username: login.username.unwrap_or_default(),
                    password: login.password.unwrap_or_default(),
                    urls: login
                        .uris
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|uri| non_empty(uri.uri))
                        .collect(),
                    notes: item.notes.unwrap_or_default(),
                    folder_path: item.folder_id.unwrap_or_default(),
                    custom_fields: convert_bitwarden_fields(item.fields),
                    source: ImportSource::Bitwarden,
                    source_item_id: item.id,
                }));
            }
            3 => {
                let Some(card) = item.card else {
                    skipped.push(SkippedImportRow {
                        row_number,
                        reason: String::from("Bitwarden card item is missing card data"),
                    });
                    continue;
                };

                items.push(ItemPayload::Card(CardPayload {
                    cardholder_name: card.cardholder_name.unwrap_or_default(),
                    number: card.number.unwrap_or_default(),
                    expiration_date: join_expiration(card.exp_month, card.exp_year),
                    security_code: card.code.unwrap_or_default(),
                    brand: card.brand.unwrap_or_default(),
                    notes: item.notes.unwrap_or_default(),
                    folder_path: item.folder_id.unwrap_or_default(),
                    custom_fields: convert_bitwarden_fields(item.fields),
                    source: ImportSource::Bitwarden,
                    source_item_id: item.id,
                }));
            }
            other => skipped.push(SkippedImportRow {
                row_number,
                reason: format!("unsupported Bitwarden item type `{other}`"),
            }),
        }
    }

    Ok((items, skipped))
}

struct HeaderMap {
    by_name: HashMap<String, usize>,
}

impl HeaderMap {
    fn new(headers: &csv::StringRecord) -> Self {
        let by_name = headers
            .iter()
            .enumerate()
            .map(|(index, header)| (normalize_header(header), index))
            .collect();

        Self { by_name }
    }

    fn get(&self, record: &csv::StringRecord, names: &[&str]) -> Option<String> {
        names.iter().find_map(|name| {
            self.by_name
                .get(&normalize_header(name))
                .and_then(|index| record.get(*index))
                .and_then(|value| non_empty(Some(value.to_owned())))
        })
    }
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonExport {
    items: Option<Vec<BitwardenJsonItem>>,
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonItem {
    id: Option<String>,
    #[serde(rename = "type")]
    item_type: u8,
    name: Option<String>,
    notes: Option<String>,
    #[serde(rename = "folderId")]
    folder_id: Option<String>,
    fields: Option<Vec<BitwardenJsonField>>,
    login: Option<BitwardenJsonLogin>,
    card: Option<BitwardenJsonCard>,
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonLogin {
    username: Option<String>,
    password: Option<String>,
    uris: Option<Vec<BitwardenJsonUri>>,
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonUri {
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonCard {
    #[serde(rename = "cardholderName")]
    cardholder_name: Option<String>,
    brand: Option<String>,
    number: Option<String>,
    #[serde(rename = "expMonth")]
    exp_month: Option<String>,
    #[serde(rename = "expYear")]
    exp_year: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitwardenJsonField {
    name: Option<String>,
    value: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<serde_json::Value>,
}

fn csv_reader(contents: &str) -> csv::Reader<&[u8]> {
    csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes())
}

fn normalize_header(header: &str) -> String {
    header
        .trim_matches('\u{feff}')
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn collect_values(values: &[Option<String>]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| value.as_ref())
        .flat_map(|value| value.split(['\n', ',']))
        .filter_map(|value| non_empty(Some(value.to_owned())))
        .collect()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();

        (!value.is_empty()).then(|| value.to_owned())
    })
}

fn join_expiration(month: Option<String>, year: Option<String>) -> String {
    match (non_empty(month), non_empty(year)) {
        (Some(month), Some(year)) => format!("{month}/{year}"),
        (Some(month), None) => month,
        (None, Some(year)) => year,
        (None, None) => String::new(),
    }
}

fn parse_bitwarden_custom_fields(fields: Option<&str>) -> Vec<CustomField> {
    fields
        .map(|fields| {
            fields
                .split('\n')
                .filter_map(|field| field.split_once(':'))
                .map(|(name, value)| CustomField {
                    name: name.trim().to_owned(),
                    value: value.trim().to_owned(),
                    field_type: String::from("text"),
                })
                .filter(|field| !field.name.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn convert_bitwarden_fields(fields: Option<Vec<BitwardenJsonField>>) -> Vec<CustomField> {
    fields
        .unwrap_or_default()
        .into_iter()
        .filter_map(|field| {
            let name = non_empty(field.name)?;
            let value = field.value.unwrap_or_default();
            let field_type = field
                .field_type
                .map(|field_type| match field_type {
                    serde_json::Value::Number(number) => number.to_string(),
                    serde_json::Value::String(value) => value,
                    other => other.to_string(),
                })
                .unwrap_or_else(|| String::from("text"));

            Some(CustomField {
                name,
                value,
                field_type,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bitwarden_csv_logins_and_cards() {
        let contents = "folder,favorite,type,name,notes,fields,login_uri,login_username,login_password,login_totp,cardholder_name,brand,number,exp_month,exp_year,code\nPersonal,0,login,GitHub,note,env: prod,https://github.com,octo,secret,,,,,,\nCards,0,card,Work Visa,,,,,,,,Jane Doe,Visa,4111111111111111,12,2030,123\n";

        let preview = parse_import(ImportFormat::BitwardenCsv, contents).unwrap();

        assert_eq!(preview.format.label(), "Bitwarden CSV");
        assert_eq!(preview.format.source().label(), "Bitwarden");
        assert_eq!(preview.imported_count(), 2);
        assert!(preview.skipped.is_empty());
        assert!(matches!(preview.items[0], ItemPayload::Login(_)));
        assert!(matches!(preview.items[1], ItemPayload::Card(_)));
    }

    #[test]
    fn parses_bitwarden_json_logins_and_cards() {
        let contents = r#"
        {
          "encrypted": false,
          "items": [
            {
              "id": "login-1",
              "type": 1,
              "name": "GitHub",
              "notes": "note",
              "login": {
                "username": "octo",
                "password": "secret",
                "uris": [{ "uri": "https://github.com" }]
              },
              "fields": [{ "name": "env", "value": "prod", "type": 0 }]
            },
            {
              "id": "card-1",
              "type": 3,
              "name": "Visa",
              "card": {
                "cardholderName": "Jane Doe",
                "brand": "Visa",
                "number": "4111111111111111",
                "expMonth": "12",
                "expYear": "2030",
                "code": "123"
              }
            }
          ]
        }
        "#;

        let preview = parse_import(ImportFormat::BitwardenJson, contents).unwrap();

        assert_eq!(preview.imported_count(), 2);
        assert!(preview.skipped.is_empty());
    }

    #[test]
    fn parses_chrome_csv() {
        let contents =
            "name,url,username,password,note\nGitHub,https://github.com,octo,secret,dev account\n";

        let preview = parse_import(ImportFormat::ChromeCsv, contents).unwrap();

        assert_eq!(preview.imported_count(), 1);
        let ItemPayload::Login(login) = &preview.items[0] else {
            panic!("expected login");
        };
        assert_eq!(login.source.label(), "Chrome");
        assert_eq!(login.urls, vec!["https://github.com"]);
    }

    #[test]
    fn parses_edge_csv() {
        let contents =
            "name,url,username,password\nExample,https://example.com,person@example.com,secret\n";

        let preview = parse_import(ImportFormat::EdgeCsv, contents).unwrap();

        assert_eq!(preview.imported_count(), 1);
        let ItemPayload::Login(login) = &preview.items[0] else {
            panic!("expected login");
        };
        assert_eq!(login.source.label(), "Edge");
    }

    #[test]
    fn parses_safari_csv() {
        let contents =
            "Title,URL,Username,Password\nExample,https://example.com,person@example.com,secret\n";

        let preview = parse_import(ImportFormat::SafariCsv, contents).unwrap();

        assert_eq!(preview.format.label(), "Safari CSV");
        assert_eq!(preview.imported_count(), 1);
        let ItemPayload::Login(login) = &preview.items[0] else {
            panic!("expected login");
        };
        assert_eq!(login.source.label(), "Safari");
        assert_eq!(login.urls, vec!["https://example.com"]);
    }

    #[test]
    fn parses_one_password_csv() {
        let contents = "Title,Website,Username,Password,Notes,Tags\nGitHub,https://github.com,octo,secret,dev,Engineering\n";

        let preview = parse_import(ImportFormat::OnePasswordCsv, contents).unwrap();

        assert_eq!(preview.imported_count(), 1);
        let ItemPayload::Login(login) = &preview.items[0] else {
            panic!("expected login");
        };
        assert_eq!(login.source.label(), "1Password");
        assert_eq!(login.folder_path, "Engineering");
    }

    #[test]
    fn parses_proton_csv() {
        let contents = "name,url,username,password,note,vault\nGitHub,https://github.com,octo,secret,dev,Personal\n";

        let preview = parse_import(ImportFormat::ProtonCsv, contents).unwrap();

        assert_eq!(preview.format.label(), "Proton CSV");
        assert_eq!(preview.imported_count(), 1);
        let ItemPayload::Login(login) = &preview.items[0] else {
            panic!("expected login");
        };
        assert_eq!(login.source.label(), "Proton");
        assert_eq!(login.urls, vec!["https://github.com"]);
        assert_eq!(login.folder_path, "Personal");
    }

    #[test]
    fn skips_blank_browser_rows() {
        let contents = "name,url,username,password\nEmpty,https://example.com,,\n";

        let preview = parse_import(ImportFormat::ChromeCsv, contents).unwrap();

        assert_eq!(preview.imported_count(), 0);
        assert_eq!(preview.skipped.len(), 1);
        assert_eq!(preview.skipped[0].row_number, 2);
        assert_eq!(preview.skipped[0].reason, "missing username and password");
    }
}
