use directories::ProjectDirs;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::crypto::{self, EncryptedPayload, MasterKey};
use crate::favicon;
use crate::models::{ItemKind, ItemPayload, Vault, VaultItem, VaultItemDetails};

const PASSWORD_CHECK: &[u8] = b"iced-vault-password-check";

pub(crate) struct Database {
    connection: Connection,
    key: MasterKey,
}

impl Database {
    pub(crate) fn needs_setup() -> Result<bool, DbError> {
        let connection = open_connection()?;
        migrate(&connection)?;

        let password_check = meta_value(&connection, "password_check_nonce")?;

        Ok(password_check.is_none())
    }

    pub(crate) fn open(master_password: &str) -> Result<Self, DbError> {
        let connection = open_connection()?;
        migrate(&connection)?;

        let salt = ensure_salt(&connection)?;
        let key = MasterKey::derive(master_password, &salt)?;

        if let Some(nonce) = meta_value(&connection, "password_check_nonce")? {
            let ciphertext = meta_value(&connection, "password_check_ciphertext")?
                .ok_or(DbError::MissingPasswordCheck)?;
            let plaintext = key
                .decrypt(&EncryptedPayload { nonce, ciphertext })
                .map_err(|error| match error {
                    crypto::CryptoError::Decrypt => DbError::InvalidMasterPassword,
                    error => DbError::Crypto(error),
                })?;

            if plaintext != PASSWORD_CHECK {
                return Err(DbError::InvalidMasterPassword);
            }
        } else {
            let encrypted = key.encrypt(PASSWORD_CHECK)?;
            connection.execute(
                "INSERT INTO app_meta (key, value) VALUES (?1, ?2)",
                params!["password_check_nonce", encrypted.nonce],
            )?;
            connection.execute(
                "INSERT INTO app_meta (key, value) VALUES (?1, ?2)",
                params!["password_check_ciphertext", encrypted.ciphertext],
            )?;
        }

        Ok(Self { connection, key })
    }

    pub(crate) fn list_vaults(&self) -> Result<Vec<Vault>, DbError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name FROM vaults ORDER BY name COLLATE NOCASE")?;
        let vaults = statement
            .query_map([], |row| {
                Ok(Vault {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(vaults)
    }

    pub(crate) fn create_vault(&self, name: &str) -> Result<Vault, DbError> {
        let vault = Vault {
            id: Uuid::new_v4().to_string(),
            name: name.trim().to_owned(),
        };
        let now = unix_timestamp();

        self.connection.execute(
            "INSERT INTO vaults (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![vault.id, vault.name, now],
        )?;

        Ok(vault)
    }

    pub(crate) fn list_items(&self, vault_id: Option<&str>) -> Result<Vec<VaultItem>, DbError> {
        let mut items = Vec::new();

        if let Some(vault_id) = vault_id {
            let mut statement = self.connection.prepare(
                "SELECT id, kind, nonce, encrypted_payload, created_at, updated_at
                 FROM items
                 WHERE vault_id = ?1
                   AND deleted_at IS NULL
                 ORDER BY updated_at DESC",
            )?;
            let rows = statement.query_map(params![vault_id], item_row)?;

            for item in rows {
                items.push(self.decrypt_item(item?)?);
            }
        }

        Ok(items)
    }

    pub(crate) fn list_items_with_payloads(
        &self,
        vault_id: &str,
    ) -> Result<Vec<VaultItemDetails>, DbError> {
        let mut statement = self.connection.prepare(
            "SELECT id, kind, nonce, encrypted_payload, created_at, updated_at
             FROM items
             WHERE vault_id = ?1
               AND deleted_at IS NULL
             ORDER BY updated_at DESC",
        )?;
        let rows = statement.query_map(params![vault_id], item_row)?;

        let mut items = Vec::new();
        for item in rows {
            items.push(self.decrypt_item_details(item?)?);
        }

        Ok(items)
    }

    pub(crate) fn create_item(
        &self,
        vault_id: &str,
        payload: ItemPayload,
        import_batch_id: Option<&str>,
    ) -> Result<VaultItem, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = unix_timestamp();
        let kind = payload.kind();
        let title = payload.title();
        let subtitle = payload.subtitle();
        let website_url = payload.website_url();
        let favicon_path = website_url
            .as_deref()
            .and_then(favicon::cached_favicon_path);
        let source_label = payload_source_label(&payload).to_owned();
        let serialized = serde_json::to_vec(&payload)?;
        let encrypted = self.key.encrypt(&serialized)?;

        self.connection.execute(
            "INSERT INTO items (
                id, vault_id, kind, source, import_batch_id, nonce, encrypted_payload, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                vault_id,
                kind.as_str(),
                source_label,
                import_batch_id,
                encrypted.nonce,
                encrypted.ciphertext,
                now,
                now,
            ],
        )?;

        Ok(VaultItem {
            id,
            kind,
            title,
            subtitle,
            updated_at: now,
            created_at: now,
            website_url,
            favicon_path,
        })
    }

    pub(crate) fn get_item(&self, item_id: &str) -> Result<Option<VaultItemDetails>, DbError> {
        let stored_item = self
            .connection
            .query_row(
                "SELECT id, kind, nonce, encrypted_payload, created_at, updated_at
                 FROM items
                 WHERE id = ?1
                   AND deleted_at IS NULL",
                params![item_id],
                item_row,
            )
            .optional()?;

        stored_item
            .map(|item| self.decrypt_item_details(item))
            .transpose()
    }

    pub(crate) fn update_item(
        &self,
        item_id: &str,
        payload: ItemPayload,
    ) -> Result<Option<VaultItemDetails>, DbError> {
        let now = unix_timestamp();
        let kind = payload.kind();
        let source_label = payload_source_label(&payload).to_owned();
        let serialized = serde_json::to_vec(&payload)?;
        let encrypted = self.key.encrypt(&serialized)?;

        self.connection.execute(
            "UPDATE items
             SET kind = ?1,
                 source = ?2,
                 nonce = ?3,
                 encrypted_payload = ?4,
                 updated_at = ?5
             WHERE id = ?6
               AND deleted_at IS NULL",
            params![
                kind.as_str(),
                source_label,
                encrypted.nonce,
                encrypted.ciphertext,
                now,
                item_id,
            ],
        )?;

        self.get_item(item_id)
    }

    pub(crate) fn delete_item(&self, item_id: &str) -> Result<(), DbError> {
        let now = unix_timestamp();

        self.connection.execute(
            "UPDATE items
             SET deleted_at = ?1,
                 updated_at = ?1
             WHERE id = ?2",
            params![now, item_id],
        )?;

        Ok(())
    }

    pub(crate) fn create_items(
        &self,
        vault_id: &str,
        payloads: Vec<ItemPayload>,
        import_batch_id: Option<&str>,
    ) -> Result<Vec<VaultItem>, DbError> {
        let transaction = self.connection.unchecked_transaction()?;
        let mut items = Vec::with_capacity(payloads.len());

        {
            let mut statement = transaction.prepare(
                "INSERT INTO items (
                    id, vault_id, kind, source, import_batch_id, nonce, encrypted_payload, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;

            for payload in payloads {
                let id = Uuid::new_v4().to_string();
                let now = unix_timestamp();
                let kind = payload.kind();
                let title = payload.title();
                let subtitle = payload.subtitle();
                let website_url = payload.website_url();
                let favicon_path = website_url
                    .as_deref()
                    .and_then(favicon::cached_favicon_path);
                let source_label = payload_source_label(&payload).to_owned();
                let serialized = serde_json::to_vec(&payload)?;
                let encrypted = self.key.encrypt(&serialized)?;

                statement.execute(params![
                    id,
                    vault_id,
                    kind.as_str(),
                    source_label,
                    import_batch_id,
                    encrypted.nonce,
                    encrypted.ciphertext,
                    now,
                    now,
                ])?;

                items.push(VaultItem {
                    id,
                    kind,
                    title,
                    subtitle,
                    created_at: now,
                    updated_at: now,
                    website_url,
                    favicon_path,
                });
            }
        }

        transaction.commit()?;

        Ok(items)
    }
}

#[derive(Debug)]
pub(crate) enum DbError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Crypto(crypto::CryptoError),
    Json(serde_json::Error),
    InvalidMasterPassword,
    MissingPasswordCheck,
    UnknownItemKind(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "database file error: {error}"),
            Self::Sql(error) => write!(formatter, "database error: {error}"),
            Self::Crypto(error) => write!(formatter, "crypto error: {error}"),
            Self::Json(error) => write!(formatter, "payload serialization error: {error}"),
            Self::InvalidMasterPassword => write!(formatter, "invalid master password"),
            Self::MissingPasswordCheck => write!(formatter, "missing password check data"),
            Self::UnknownItemKind(kind) => write!(formatter, "unknown item kind: {kind}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

impl From<crypto::CryptoError> for DbError {
    fn from(error: crypto::CryptoError) -> Self {
        Self::Crypto(error)
    }
}

impl From<serde_json::Error> for DbError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

struct StoredItem {
    id: String,
    kind: String,
    nonce: Vec<u8>,
    encrypted_payload: Vec<u8>,
    created_at: i64,
    updated_at: i64,
}

fn open_connection() -> Result<Connection, DbError> {
    let path = ProjectDirs::from("dev", "iced-vault", "iced-vault")
        .map(|directories| directories.data_dir().join("vault.sqlite3"))
        .unwrap_or_else(|| std::env::temp_dir().join("iced-vault.sqlite3"));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    Ok(connection)
}

fn migrate(connection: &Connection) -> Result<(), DbError> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS vaults (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS import_batches (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            source_file_name TEXT,
            source_format TEXT,
            imported_at INTEGER NOT NULL,
            item_count INTEGER NOT NULL DEFAULT 0,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            vault_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'Manual',
            source_item_ref TEXT,
            import_batch_id TEXT,
            nonce BLOB NOT NULL,
            encrypted_payload BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER,
            FOREIGN KEY (vault_id) REFERENCES vaults(id) ON DELETE CASCADE,
            FOREIGN KEY (import_batch_id) REFERENCES import_batches(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_items_vault_updated_at ON items(vault_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_items_import_batch ON items(import_batch_id);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_items_source_ref
            ON items(vault_id, source, source_item_ref)
            WHERE source_item_ref IS NOT NULL;
        ",
    )?;

    Ok(())
}

fn ensure_salt(connection: &Connection) -> Result<Vec<u8>, DbError> {
    let existing = meta_value(connection, "kdf_salt")?;

    if let Some(salt) = existing {
        return Ok(salt);
    }

    let salt = crypto::random_salt();
    connection.execute(
        "INSERT INTO app_meta (key, value) VALUES ('kdf_salt', ?1)",
        params![salt],
    )?;

    Ok(salt)
}

fn meta_value(connection: &Connection, key: &str) -> Result<Option<Vec<u8>>, DbError> {
    Ok(connection
        .query_row(
            "SELECT value FROM app_meta WHERE key = ?1",
            params![key],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?)
}

fn item_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredItem> {
    Ok(StoredItem {
        id: row.get(0)?,
        kind: row.get(1)?,
        nonce: row.get(2)?,
        encrypted_payload: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

impl Database {
    fn decrypt_item(&self, item: StoredItem) -> Result<VaultItem, DbError> {
        let payload_bytes = self.key.decrypt(&EncryptedPayload {
            nonce: item.nonce,
            ciphertext: item.encrypted_payload,
        })?;
        let payload: ItemPayload = serde_json::from_slice(&payload_bytes)?;
        let kind = ItemKind::from_str(&item.kind).ok_or(DbError::UnknownItemKind(item.kind))?;

        Ok(VaultItem {
            id: item.id,
            kind,
            title: payload.title(),
            subtitle: payload.subtitle(),
            created_at: item.created_at,
            updated_at: item.updated_at,
            website_url: payload.website_url(),
            favicon_path: payload
                .website_url()
                .as_deref()
                .and_then(favicon::cached_favicon_path),
        })
    }

    fn decrypt_item_details(&self, item: StoredItem) -> Result<VaultItemDetails, DbError> {
        let payload_bytes = self.key.decrypt(&EncryptedPayload {
            nonce: item.nonce,
            ciphertext: item.encrypted_payload,
        })?;
        let payload: ItemPayload = serde_json::from_slice(&payload_bytes)?;
        let kind = ItemKind::from_str(&item.kind).ok_or(DbError::UnknownItemKind(item.kind))?;
        let website_url = payload.website_url();
        let favicon_path = website_url
            .as_deref()
            .and_then(favicon::cached_favicon_path);

        Ok(VaultItemDetails {
            id: item.id,
            kind,
            title: payload.title(),
            subtitle: payload.subtitle(),
            created_at: item.created_at,
            updated_at: item.updated_at,
            website_url,
            favicon_path,
            payload,
        })
    }
}

fn payload_source_label(payload: &ItemPayload) -> &str {
    match payload {
        ItemPayload::Login(login) => login.source.label(),
        ItemPayload::Card(card) => card.source.label(),
    }
}

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

impl Database {
    pub(crate) fn change_master_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), DbError> {
        let salt = ensure_salt(&self.connection)?;
        let old_key = MasterKey::derive(old_password, &salt)?;
        let password_check = EncryptedPayload {
            nonce: meta_value(&self.connection, "password_check_nonce")?
                .ok_or(DbError::MissingPasswordCheck)?,
            ciphertext: meta_value(&self.connection, "password_check_ciphertext")?
                .ok_or(DbError::MissingPasswordCheck)?,
        };

        let password_check = old_key
            .decrypt(&password_check)
            .map_err(|error| match error {
                crypto::CryptoError::Decrypt => DbError::InvalidMasterPassword,
                error => DbError::Crypto(error),
            })?;

        if password_check != PASSWORD_CHECK {
            return Err(DbError::InvalidMasterPassword);
        }

        let new_key = MasterKey::derive(new_password, &salt)?;
        let transaction = self.connection.unchecked_transaction()?;

        let items: Vec<(String, Vec<u8>, Vec<u8>)> = {
            let mut statement =
                transaction.prepare("SELECT id, nonce, encrypted_payload FROM items")?;
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<Result<Vec<_>, _>>()?
        };

        for (id, old_nonce, old_ciphertext) in items {
            let encrypted = EncryptedPayload {
                nonce: old_nonce,
                ciphertext: old_ciphertext,
            };
            let plaintext = old_key
                .decrypt(&encrypted)
                .map_err(|_| DbError::InvalidMasterPassword)?;

            let new_encrypted = new_key.encrypt(&plaintext)?;

            transaction.execute(
                "UPDATE items SET nonce = ?1, encrypted_payload = ?2 WHERE id = ?3",
                params![new_encrypted.nonce, new_encrypted.ciphertext, id],
            )?;
        }

        let new_encrypted = new_key.encrypt(PASSWORD_CHECK)?;
        transaction.execute(
            "UPDATE app_meta SET value = ?1 WHERE key = 'password_check_nonce'",
            params![new_encrypted.nonce],
        )?;
        transaction.execute(
            "UPDATE app_meta SET value = ?1 WHERE key = 'password_check_ciphertext'",
            params![new_encrypted.ciphertext],
        )?;
        transaction.commit()?;

        Ok(())
    }

    pub(crate) fn set_auto_lock_duration(&self, duration: i64) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT OR REPLACE INTO app_meta (key, value) VALUES ('auto_lock_duration', ?1)",
            params![duration.to_string()],
        )?;
        Ok(())
    }

    pub(crate) fn get_auto_lock_duration(&self) -> Result<i64, DbError> {
        let result: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'auto_lock_duration'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(result.and_then(|s| s.parse().ok()).unwrap_or(600))
    }
}
