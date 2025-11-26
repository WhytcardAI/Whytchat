use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::fs_manager::PortablePathManager;

const NONCE_SIZE: usize = 12;

/// Global cached encryption key (generated once at startup)
static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();

/// Gets the path to the key file in the data directory
fn get_key_file_path() -> PathBuf {
    PortablePathManager::data_dir().join(".encryption_key")
}

/// Generates a cryptographically secure 32-byte key
fn generate_secure_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
}

/// Retrieves the 32-byte encryption key.
///
/// Priority order:
/// 1. Cached key (if already loaded this session)
/// 2. ENCRYPTION_KEY environment variable (for testing/CI)
/// 3. Stored key file in data directory
/// 4. Generate new key and save it
///
/// # Returns
///
/// A `Result` containing the 32-byte key array, or an error `String` on failure.
pub fn get_encryption_key() -> Result<[u8; 32], String> {
    // Return cached key if available
    if let Some(key) = ENCRYPTION_KEY.get() {
        return Ok(*key);
    }

    // Try environment variable first (for testing/CI)
    if let Ok(key_str) = env::var("ENCRYPTION_KEY") {
        let mut key_bytes = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key_bytes[..len].copy_from_slice(&bytes[..len]);
        let _ = ENCRYPTION_KEY.set(key_bytes);
        return Ok(key_bytes);
    }

    // Try to load from file
    let key_path = get_key_file_path();

    let key = if key_path.exists() {
        // Load existing key
        let key_b64 = fs::read_to_string(&key_path)
            .map_err(|e| format!("Failed to read encryption key file: {}", e))?;

        let key_bytes = general_purpose::STANDARD
            .decode(key_b64.trim())
            .map_err(|e| format!("Failed to decode encryption key: {}", e))?;

        if key_bytes.len() != 32 {
            return Err(format!(
                "Invalid key length: expected 32, got {}",
                key_bytes.len()
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        key
    } else {
        // Generate new key and save it
        let key = generate_secure_key();

        // Ensure parent directory exists
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        // Save key as base64
        let key_b64 = general_purpose::STANDARD.encode(key);
        fs::write(&key_path, &key_b64)
            .map_err(|e| format!("Failed to save encryption key: {}", e))?;

        // Set file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&key_path, perms);
        }

        key
    };

    // Cache the key
    let _ = ENCRYPTION_KEY.set(key);
    Ok(key)
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
        temp_env::with_var(
            "ENCRYPTION_KEY",
            Some("01234567890123456789012345678901"),
            || {
                let data = b"Sensitive Data";
                let encrypted = encrypt(data).expect("Encryption failed");
                let decrypted = decrypt(&encrypted).expect("Decryption failed");
                assert_eq!(data, &decrypted[..]);
            },
        );
    }
}
