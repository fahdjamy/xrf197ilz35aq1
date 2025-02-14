use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use crate::core::DomainError;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use getrandom;
use ring::rand::SecureRandom;
use sha2::{Digest, Sha512};
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct NFC {
    pub id: String,
    pub cert: String,
    pub asset_id: String,
    pub created_at: DateTime<Utc>,
}

impl NFC {
    pub fn new(asset_id: String) -> Result<Self, DomainError> {
        let cert_id = generate_unique_key(DOMAIN_KEY_SIZE);
        let certificate = generate_certificate(&asset_id)
            .map_err(|err| {
                return DomainError::ServerError(format!("Failed to generate certificate: {}", err));
            })?;
        let now = Utc::now();
        Ok(Self {
            asset_id,
            id: cert_id,
            created_at: now,
            cert: certificate,
        })
    }
}

impl Display for NFC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Certificate: Id: {}, AssetId: {}, Created At: {}",
               self.id, self.asset_id, self.created_at)
    }
}

/// Generates a unique, non-fungible string.
///
/// This function combines multiple sources of entropy, including:
/// 1.  A high-precision timestamp (nanoseconds).
/// 2.  Random bytes from a cryptographically secure random bytes generator (using ring).
/// 3.  A thread-safe, globally unique counter (incremented for each call).
///
/// The combined data is then hashed using SHA-256, and the resulting hash is encoded
/// using Base64 URL-safe encoding to produce the final string.
///
/// # Thread Safety
///
/// The `GLOBAL_NFT_COUNTER` is protected by an `Arc<Mutex<>>`, ensuring that only one thread
/// can increment the counter at a time, preventing race conditions and guaranteeing
/// uniqueness across multiple threads.
///
/// # Security Considerations
///
/// *   **ring:** Use the ring CSPRNG for strong random bytes' generation.
/// *   **SHA-256:** Employs the SHA-256 hashing algorithm for collision resistance.  While SHA-256
///     is currently considered secure, it is recommended to monitor cryptographic best practices and
///     potentially migrate to stronger hashing algorithms (like SHA-3) as they become available.
/// *   **Timestamp Precision:** Uses nanosecond-precision timestamps for increased entropy.
///     This reduces (but does not completely eliminate) the tiny risk of collision
///     if two tokens are generated within the same nanosecond on different threads, *and* the
///     random bytes happen to be identical (extraordinarily unlikely).
/// * **Global Counter:** The global counter adds process-wide (or even machine wide if combined w/
///     a unique machine identifier) uniqueness.
/// * **Base64 Encoding:** Base64 URL-safe encoding makes the generated string suitable for use in
///     URLs and other contexts where special characters are not allowed.
fn generate_certificate(asset_id: &str) -> Result<String, String> {
    // Use an Arc<Mutex<>> to make the counter thread-safe.
    lazy_static::lazy_static! {
        static ref GLOBAL_NFT_COUNTER: Arc<Mutex<u128>> = Arc::new(Mutex::new(0));
    }

    // 1. Get a high-precision timestamp (nanoseconds).
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| {
            return format!("Failed to get system date: err={}", err);
        });
    let timestamp_nanos = now?.as_nanos();

    // 2. Generate secure random bytes using ring.
    let mut random_bytes = vec![0u8; 64];
    let rng = ring::rand::SystemRandom::new();
    rng.fill(&mut random_bytes).map_err(|err| {
        return format!("Failed to generate random bytes with ring: err={}", err);
    })?;

    let mut salt = vec![0u8; 32];
    let rng = ring::rand::SystemRandom::new();
    rng.fill(&mut salt).map_err(|err| {
        return format!("Failed to generate random bytes with for salt: err={}", err);
    })?;

    let mut counter_guard = GLOBAL_NFT_COUNTER
        .lock()
        .map_err(|err| {
            return format!("Failed to lock global nft: err={}", err);
        })?;

    let counter_val = *counter_guard;
    *counter_guard += 1;

    // 2. Combine timestamp, random bytes, and asset_id.
    let salt = generate_salt(asset_id)?; // Salt to make data more unique
    let combined_data = format!(
        "{}*{}**{}*{}",
        timestamp_nanos,
        hex::encode(random_bytes), //hex encode for better formatting
        counter_val,
        salt
    );

    // 3. Hash using SHA-256 (SHA-512 is slow and best for 64-bit OS).
    let mut hasher = Sha512::new();
    hasher.update(combined_data);
    let hash_result = hasher.finalize();

    // 4. encode using Base64 for URL-safety.
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(hash_result))
}

fn generate_salt(asset_id: &str) -> Result<String, String> {
    let mut salt_bytes = [0u8; 16];
    getrandom::fill(&mut salt_bytes).map_err(|err| {
        return format!("Failed to generate salt: err={}", err);
    })?;
    Ok(format!("{}_{}", asset_id, general_purpose::URL_SAFE_NO_PAD.encode(salt_bytes)))
}

#[cfg(test)]
mod tests {
    use crate::core::domain::nfc::generate_certificate;
    use std::collections::HashSet;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn test_generate_certificate() {
        let asset_id = "1234";
        let cert_1 = generate_certificate(asset_id);
        assert!(cert_1.is_ok());
        let cert_2 = generate_certificate(asset_id);
        assert!(cert_2.is_ok());
        assert_ne!(cert_1, cert_2, "IDs should be different");
    }

    #[test]
    fn test_thread_safety() {
        let num_threads = 10;
        let asset_id = "1234";
        let ids_per_thread = 100;
        let (tx, rx) = mpsc::channel();

        for _ in 0..num_threads {
            let tx = tx.clone();
            thread::spawn(move || {
                for _ in 0..ids_per_thread {
                    let cert_id = generate_certificate(asset_id);
                    assert!(cert_id.is_ok());
                    tx.send(cert_id.unwrap()).unwrap();
                }
            });
        }

        let mut all_ids = HashSet::new();
        for _ in 0..(num_threads * ids_per_thread) {
            let id = rx.recv().unwrap();
            assert!(!all_ids.contains(&id), "Duplicate ID found: {}", id);
            all_ids.insert(id);
        }
        assert_eq!(all_ids.len(), num_threads * ids_per_thread);
    }
}
