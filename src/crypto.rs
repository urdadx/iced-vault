use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand_core::{OsRng, RngCore};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug)]
pub(crate) enum CryptoError {
    InvalidSaltLength,
    InvalidNonceLength,
    KeyDerivation(argon2::Error),
    Encrypt,
    Decrypt,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSaltLength => write!(formatter, "invalid encryption salt length"),
            Self::InvalidNonceLength => write!(formatter, "invalid encryption nonce length"),
            Self::KeyDerivation(error) => write!(formatter, "key derivation failed: {error}"),
            Self::Encrypt => write!(formatter, "encryption failed"),
            Self::Decrypt => write!(formatter, "decryption failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[derive(Debug, Clone)]
pub(crate) struct MasterKey([u8; KEY_LEN]);

impl MasterKey {
    pub(crate) fn derive(password: &str, salt: &[u8]) -> Result<Self, CryptoError> {
        let salt: [u8; SALT_LEN] = salt
            .try_into()
            .map_err(|_| CryptoError::InvalidSaltLength)?;
        let params =
            Params::new(64 * 1024, 3, 1, Some(KEY_LEN)).map_err(CryptoError::KeyDerivation)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0; KEY_LEN];

        argon2
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(CryptoError::KeyDerivation)?;

        Ok(Self(key))
    }

    pub(crate) fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedPayload, CryptoError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce = random_nonce();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|_| CryptoError::Encrypt)?;

        Ok(EncryptedPayload { nonce, ciphertext })
    }

    pub(crate) fn decrypt(&self, payload: &EncryptedPayload) -> Result<Vec<u8>, CryptoError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce: [u8; NONCE_LEN] = payload
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidNonceLength)?;

        cipher
            .decrypt(Nonce::from_slice(&nonce), payload.ciphertext.as_ref())
            .map_err(|_| CryptoError::Decrypt)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EncryptedPayload {
    pub(crate) nonce: Vec<u8>,
    pub(crate) ciphertext: Vec<u8>,
}

pub(crate) fn random_salt() -> Vec<u8> {
    let mut salt = vec![0; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn random_nonce() -> Vec<u8> {
    let mut nonce = vec![0; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
