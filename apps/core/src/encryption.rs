use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use std::env;

const NONCE_SIZE: usize = 12;

/// Retrieves the 32-byte encryption key from the `ENCRYPTION_KEY` environment variable.
///
/// This function reads the environment variable and converts it into a fixed-size 32-byte array.
/// If the variable is shorter than 32 bytes, it's padded with zeros. If it's longer, it's truncated.
///
/// # Returns
///
/// A `Result` containing the 32-byte key array, or an error `String` if the environment
/// variable is not set.
pub fn get_encryption_key() -> Result<[u8; 32], String> {
    let key_str = env::var("ENCRYPTION_KEY").map_err(|_| "ENCRYPTION_KEY environment variable not set")?;

    // We expect a 32-byte key.
    // For simplicity, we pad with zeros if shorter, and truncate if longer.
    // In a production environment, this should be handled more robustly (e.g., hex decoding).
    let mut key_bytes = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = bytes.len().min(32);
    key_bytes[..len].copy_from_slice(&bytes[..len]);

    Ok(key_bytes)
}

/// Encrypts data using AES-256-GCM.
///
/// A random 12-byte nonce is generated for each encryption. The nonce is prepended
/// to the ciphertext, and the combined result is Base64 encoded.
///
/// # Arguments
///
/// * `data` - A slice of bytes to be encrypted.
///
/// # Returns
///
/// A `Result` containing the Base64-encoded ciphertext (nonce + encrypted data),
/// or an error `String` on failure.
pub fn encrypt(data: &[u8]) -> Result<String, String> {
    let key_bytes = get_encryption_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut rng = rand::thread_rng();
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failure: {}", e))?;

    // On concatène le nonce et le ciphertext
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

/// Decrypts data that was encrypted with `encrypt`.
///
/// This function decodes the Base64 input, separates the nonce from the ciphertext,
/// and performs AES-256-GCM decryption.
///
/// # Arguments
///
/// * `encrypted_base64` - The Base64-encoded string containing the nonce and ciphertext.
///
/// # Returns
///
/// A `Result` containing the decrypted data as a `Vec<u8>`, or an error `String` on failure.
pub fn decrypt(encrypted_base64: &str) -> Result<Vec<u8>, String> {
    let key_bytes = get_encryption_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let decoded = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| format!("Base64 decode failure: {}", e))?;

    if decoded.len() < NONCE_SIZE {
        return Err("Invalid encrypted data length".to_string());
    }

    let nonce = Nonce::from_slice(&decoded[..NONCE_SIZE]);
    let ciphertext = &decoded[NONCE_SIZE..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failure: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        temp_env::with_var("ENCRYPTION_KEY", Some("01234567890123456789012345678901"), || {
            let data = b"Sensitive Data";
            let encrypted = encrypt(data).expect("Encryption failed");
            let decrypted = decrypt(&encrypted).expect("Decryption failed");
            assert_eq!(data, &decrypted[..]);
        });
    }
}