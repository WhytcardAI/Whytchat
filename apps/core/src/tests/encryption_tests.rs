//! Encryption Module Tests
//!
//! Tests for AES-256-GCM encryption/decryption functionality.

use crate::encryption;

#[cfg(test)]
mod encryption_tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "Secret message that needs encryption";

        let encrypted = encryption::encrypt(original.as_bytes())
            .expect("Encryption should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(
            String::from_utf8(decrypted).unwrap(),
            original
        );
    }

    #[test]
    fn test_encrypt_empty_data() {
        let empty = "";

        let encrypted = encryption::encrypt(empty.as_bytes())
            .expect("Encryption of empty data should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_encrypt_long_data() {
        let long_data = "A".repeat(10000);

        let encrypted = encryption::encrypt(long_data.as_bytes())
            .expect("Encryption should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(
            String::from_utf8(decrypted).unwrap(),
            long_data
        );
    }

    #[test]
    fn test_encrypt_binary_data() {
        let binary_data: Vec<u8> = (0..=255).collect();

        let encrypted = encryption::encrypt(&binary_data)
            .expect("Encryption should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(decrypted, binary_data);
    }

    #[test]
    fn test_encrypt_unicode() {
        let unicode = "Hello 世界! 🌍 Привет мир! مرحبا بالعالم";

        let encrypted = encryption::encrypt(unicode.as_bytes())
            .expect("Encryption should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(
            String::from_utf8(decrypted).unwrap(),
            unicode
        );
    }

    #[test]
    fn test_encrypted_data_is_different() {
        let original = "Test data";

        let encrypted1 = encryption::encrypt(original.as_bytes())
            .expect("Encryption should succeed");
        let encrypted2 = encryption::encrypt(original.as_bytes())
            .expect("Encryption should succeed");

        // Due to random nonce, encrypted data should be different each time
        assert_ne!(
            encrypted1, encrypted2,
            "Each encryption should produce different ciphertext due to random nonce"
        );
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let invalid = "not-valid-base64-encrypted-data!!!";

        let result = encryption::decrypt(invalid);

        assert!(result.is_err(), "Decryption of invalid data should fail");
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let original = "Original message";

        let mut encrypted = encryption::encrypt(original.as_bytes())
            .expect("Encryption should succeed");

        // Tamper with the encrypted data
        if encrypted.len() > 10 {
            encrypted.replace_range(5..8, "XXX");
        }

        let result = encryption::decrypt(&encrypted);

        assert!(result.is_err(), "Decryption of tampered data should fail");
    }

    #[test]
    fn test_json_encryption() {
        use serde::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Config {
            api_key: String,
            secret: String,
        }

        let config = Config {
            api_key: "my-api-key".to_string(),
            secret: "super-secret-value".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();

        let encrypted = encryption::encrypt(json.as_bytes())
            .expect("Encryption should succeed");

        let decrypted = encryption::decrypt(&encrypted)
            .expect("Decryption should succeed");

        let restored: Config = serde_json::from_slice(&decrypted).unwrap();

        assert_eq!(restored, config);
    }
}
