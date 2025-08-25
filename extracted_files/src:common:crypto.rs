// Shared cryptographic utilities
use sha2::{Sha256, Digest};

pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn calculate_merkle_root(items: &[String]) -> String {
    if items.is_empty() {
        return "0".repeat(64);
    }
    
    let mut hasher = Sha256::new();
    for item in items {
        hasher.update(item.as_bytes());
    }
    hex::encode(hasher.finalize())
}

pub fn verify_proof_of_work(hash: &str, difficulty: u32) -> bool {
    let target = "0".repeat(difficulty as usize);
    hash.starts_with(&target)
}
