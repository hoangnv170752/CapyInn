use sha2::{Sha256, Digest};
use sqlx::{Pool, Sqlite};
use rand::Rng;

const KEY_PREFIX: &str = "hmg_sk_";
const KEY_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Generate a new API key pair: (plaintext key, sha256 hash)
pub fn generate_api_key() -> (String, String) {
    let mut rng = rand::thread_rng();
    let random_part: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..KEY_CHARS.len());
            KEY_CHARS[idx] as char
        })
        .collect();

    let key = format!("{}{}", KEY_PREFIX, random_part);
    let hash = hash_key(&key);
    (key, hash)
}

/// SHA-256 hash of a key
pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Store a hashed API key in the database
pub async fn store_api_key(pool: &Pool<Sqlite>, key_hash: &str, label: &str) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO gateway_api_keys (id, key_hash, label, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(key_hash)
    .bind(label)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to store API key: {}", e))?;

    Ok(id)
}

/// Validate an API key against stored hashes. Returns true if valid.
pub async fn validate_api_key(pool: &Pool<Sqlite>, key: &str) -> bool {
    if !key.starts_with(KEY_PREFIX) {
        return false;
    }

    let key_hash = hash_key(key);

    let result: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM gateway_api_keys WHERE key_hash = ?"
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some((id,)) = result {
        // Update last_used_at
        let now = chrono::Local::now().to_rfc3339();
        let _ = sqlx::query("UPDATE gateway_api_keys SET last_used_at = ? WHERE id = ?")
            .bind(&now)
            .bind(&id)
            .execute(pool)
            .await;
        true
    } else {
        false
    }
}

/// Check if any API keys exist
pub async fn has_api_keys(pool: &Pool<Sqlite>) -> bool {
    let count: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM gateway_api_keys")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    count.map(|c| c.0 > 0).unwrap_or(false)
}
