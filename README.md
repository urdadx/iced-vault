# iced vault

`iced-vault` is a local desktop password manager built with Rust and `iced`.
It stores vault data in a local SQLite database and encrypts item payloads at rest with a master-password-derived key.

## Features

- Local vault creation and unlock flow with a master password
- Encrypted storage for login and card items
- Vault browsing and item detail views
- Import support for Bitwarden, Chrome, Edge, 1Password, Proton, and Safari exports
- Export support for CSV, ZIP, and the current PGP-labeled export flow in Settings
- Auto-lock with configurable timeout
- Master password rotation
- Light and dark theme toggle

## Demo


https://github.com/user-attachments/assets/775966af-8b87-40c6-8dd7-214febc15857



## Security Notes

- Vault data is encrypted at rest using `ChaCha20-Poly1305`
- The master key is derived from the master password using `Argon2id`
- A password verifier is stored so the app can validate the master password without storing it directly
- Data is stored locally only; there is no sync or remote backend in this repository

## Storage

The application stores its SQLite database in the platform-specific app data directory managed by the `directories` crate.
The database file name is `vault.sqlite3`.

## Supported Imports

- Bitwarden CSV
- Bitwarden JSON
- Chrome CSV
- Edge CSV
- 1Password CSV
- Proton CSV
- Safari CSV

Imported items currently map to these local item types:

- Login
- Card

## Supported Exports

- CSV
- ZIP containing a CSV export
- A text export path exposed in the UI as `PGP-encrypted`

## Running Locally

Requirements:

- Rust toolchain with Cargo

Run the application:

```bash
cargo run
```

Check the project builds:

```bash
cargo check
```

## Tech Stack

- Rust
- `iced` for the desktop UI
- `rusqlite` with bundled SQLite
- `argon2` and `chacha20poly1305` from RustCrypto
- `rfd` for native file dialogs

## Current Limitations
- This project is local-first and does not include sync, sharing, or cloud backup
