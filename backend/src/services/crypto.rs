use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

use crate::messages::error;

const NONCE_SIZE: usize = 12;

pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("{}: {}", error::CIPHER_CREATE_FAILED, e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("{}: {}", error::ENCRYPTION_FAILED, e))?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&result))
}

pub fn decrypt(encrypted: &str, key: &[u8; 32]) -> Result<String, String> {
    let data = BASE64
        .decode(encrypted)
        .map_err(|e| format!("{}: {}", error::BASE64_DECODE_FAILED, e))?;

    if data.len() < NONCE_SIZE {
        return Err(error::INVALID_ENCRYPTED_DATA.to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("{}: {}", error::CIPHER_CREATE_FAILED, e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("{}: {}", error::DECRYPTION_FAILED, e))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("{}: {}", error::INVALID_UTF8, e))
}

pub fn derive_key(secret: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = derive_key("test-secret-key-for-testing");
        let plaintext = "my-secret-token-12345";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
