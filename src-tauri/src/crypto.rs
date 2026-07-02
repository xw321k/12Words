use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::OsRng;
use bip39::{Mnemonic};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Generate a 12-word BIP39 mnemonic
pub fn generate_mnemonic() -> Result<String, String> {
    let mnemonic = Mnemonic::generate(12).map_err(|e| format!("Failed to generate mnemonic: {}", e))?;
    Ok(mnemonic.to_string())
}

/// Validate a 12-word mnemonic string
pub fn validate_mnemonic(phrase: &str) -> Result<(), String> {
    match Mnemonic::parse(phrase) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Invalid mnemonic: {}", e)),
    }
}

/// Convert mnemonic to 512-bit seed (BIP39 standard)
pub fn mnemonic_to_seed(phrase: &str) -> Result<String, String> {
    let mnemonic = Mnemonic::parse(phrase)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    let seed = mnemonic.to_seed("");
    Ok(hex::encode(seed))
}

/// Derive a UserID from seed (SHA256 of seed → cannot reverse to seed)
pub fn derive_user_id(seed_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed_hex.as_bytes());
    hex::encode(hasher.finalize())
}

/// Derive AES-256 key (32 bytes) from the 512-bit seed
/// Uses first 32 bytes of the seed as the encryption key
fn derive_encryption_key(seed_hex: &str) -> Result<[u8; 32], String> {
    let seed_bytes = hex::decode(seed_hex).map_err(|e| format!("Invalid seed hex: {}", e))?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&seed_bytes[..32]);
    Ok(key)
}

/// Encrypt data using AES-256-GCM with a key derived from seed
pub fn encrypt_vault(seed_hex: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = derive_encryption_key(seed_hex)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    // Generate random 96-bit nonce
    let nonce_bytes: [u8; 12] = OsRng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext (nonce + ciphertext + tag)
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_vault(seed_hex: &str, encrypted: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted.len() < 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let key = derive_encryption_key(seed_hex)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Vault data structure that gets encrypted/decrypted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub note: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultData {
    pub entries: Vec<VaultEntry>,
}

impl VaultData {
    pub fn empty() -> Self {
        VaultData {
            entries: Vec::new(),
        }
    }

    pub fn to_encrypted(&self, seed_hex: &str) -> Result<Vec<u8>, String> {
        let json = serde_json::to_vec(self).map_err(|e| format!("Serialization failed: {}", e))?;
        encrypt_vault(seed_hex, &json)
    }

    pub fn from_encrypted(seed_hex: &str, data: &[u8]) -> Result<Self, String> {
        let json = decrypt_vault(seed_hex, data)?;
        serde_json::from_slice(&json).map_err(|e| format!("Deserialization failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let phrase = generate_mnemonic().unwrap();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_mnemonic_roundtrip() {
        let phrase = generate_mnemonic().unwrap();
        assert!(validate_mnemonic(&phrase).is_ok());
    }

    #[test]
    fn test_seed_derivation() {
        let phrase = generate_mnemonic().unwrap();
        let seed = mnemonic_to_seed(&phrase).unwrap();
        // BIP39 seed is 512 bits = 128 hex chars
        assert_eq!(seed.len(), 128);
    }

    #[test]
    fn test_vault_encrypt_decrypt() {
        let phrase = generate_mnemonic().unwrap();
        let seed = mnemonic_to_seed(&phrase).unwrap();

        let vault = VaultData {
            entries: vec![VaultEntry {
                id: "1".to_string(),
                title: "Test".to_string(),
                url: "https://example.com".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                note: "note".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            }],
        };

        let encrypted = vault.to_encrypted(&seed).unwrap();
        let decrypted = VaultData::from_encrypted(&seed, &encrypted).unwrap();

        assert_eq!(decrypted.entries.len(), 1);
        assert_eq!(decrypted.entries[0].username, "user");
    }

    #[test]
    fn test_derive_user_id() {
        let phrase = generate_mnemonic().unwrap();
        let seed = mnemonic_to_seed(&phrase).unwrap();
        let user_id = derive_user_id(&seed);
        // SHA256 = 64 hex chars
        assert_eq!(user_id.len(), 64);
    }
}
