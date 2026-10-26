use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption key must be 32 bytes")]
    InvalidKeyLength,
    #[error("Encryption failed: {0}")]
    EncryptFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptFailed(String),
    #[error("Ciphertext too short")]
    CiphertextTooShort,
}

/// Encrypts plaintext using AES-256-GCM
/// Format: nonce (12 bytes) + ciphertext (matches Go implementation)
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKeyLength);
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptFailed(e.to_string()))?;

    // Prepend nonce to ciphertext (matches Go format)
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts ciphertext using AES-256-GCM
/// Expects format: nonce (12 bytes) + ciphertext
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKeyLength);
    }

    if ciphertext.len() < 12 {
        return Err(CryptoError::CiphertextTooShort);
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptFailed(e.to_string()))?;

    let (nonce_bytes, ciphertext) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptFailed(e.to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = b"01234567890123456789012345678901"; // 32 bytes
        let plaintext = b"test secret data";

        let ciphertext = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&ciphertext, key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_invalid_key_length() {
        let key = b"short";
        let plaintext = b"test";
        let result = encrypt(plaintext, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_key_length() {
        let key = b"short";
        let ciphertext = vec![0u8; 32];
        let result = decrypt(&ciphertext, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_ciphertext_too_short() {
        let key = b"01234567890123456789012345678901";
        let ciphertext = vec![0u8; 5]; // Less than 12 bytes
        let result = decrypt(&ciphertext, key);
        assert!(result.is_err());
    }
}
