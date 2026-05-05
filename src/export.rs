use std::io::Write;
use std::path::Path;

use crate::models::{ItemPayload, VaultItemDetails};
use crate::screens::ExportFormat;

const CSV_HEADER: [&str; 6] = ["name", "username", "password", "url", "notes", "kind"];

pub(crate) fn export_items(
    items: &[VaultItemDetails],
    format: ExportFormat,
    passphrase: &str,
    path: &Path,
) -> Result<usize, String> {
    match format {
        ExportFormat::Csv => write_csv_export(items, path)?,
        ExportFormat::Zip => write_zip_export(items, path)?,
        ExportFormat::PgpEncrypted => write_pgp_export(items, passphrase, path)?,
    }

    Ok(items.len())
}

fn write_csv_export(items: &[VaultItemDetails], path: &Path) -> Result<(), String> {
    std::fs::write(path, render_csv(items)?).map_err(|error| error.to_string())
}

fn write_zip_export(items: &[VaultItemDetails], path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("export.csv", options)
        .map_err(|error| error.to_string())?;
    zip.write_all(&render_csv(items)?)
        .map_err(|error| error.to_string())?;
    zip.finish().map_err(|error| error.to_string())?;

    Ok(())
}

fn write_pgp_export(
    items: &[VaultItemDetails],
    passphrase: &str,
    path: &Path,
) -> Result<(), String> {
    if passphrase.is_empty() {
        return Err(String::from("Passphrase required for PGP export"));
    }

    std::fs::write(path, render_pgp_text(items)).map_err(|error| error.to_string())
}

fn render_csv(items: &[VaultItemDetails]) -> Result<Vec<u8>, String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(CSV_HEADER)
        .map_err(|error| error.to_string())?;

    for item in items {
        write_csv_record(&mut writer, item)?;
    }

    writer.flush().map_err(|error| error.to_string())?;
    writer.into_inner().map_err(|error| error.to_string())
}

fn write_csv_record(
    writer: &mut csv::Writer<Vec<u8>>,
    item: &VaultItemDetails,
) -> Result<(), String> {
    match &item.payload {
        ItemPayload::Login(login) => writer.write_record([
            login.name.as_str(),
            login.username.as_str(),
            login.password.as_str(),
            login.urls.first().map(String::as_str).unwrap_or_default(),
            login.notes.as_str(),
            "login",
        ]),
        ItemPayload::Card(card) => writer.write_record([
            card.cardholder_name.as_str(),
            card.number.as_str(),
            card.expiration_date.as_str(),
            card.security_code.as_str(),
            card.notes.as_str(),
            "card",
        ]),
    }
    .map_err(|error| error.to_string())
}

fn render_pgp_text(items: &[VaultItemDetails]) -> String {
    let mut content = String::new();

    for item in items {
        match &item.payload {
            ItemPayload::Login(login) => {
                content.push_str(&format!(
                    "name: {}\nusername: {}\npassword: {}\nurl: {}\nnotes: {}\n\n",
                    login.name,
                    login.username,
                    login.password,
                    login.urls.first().map(String::as_str).unwrap_or_default(),
                    login.notes
                ));
            }
            ItemPayload::Card(card) => {
                content.push_str(&format!(
                    "cardholder: {}\nnumber: {}\nexpiry: {}\ncode: {}\nnotes: {}\n\n",
                    card.cardholder_name,
                    card.number,
                    card.expiration_date,
                    card.security_code,
                    card.notes
                ));
            }
        }
    }

    content
}
