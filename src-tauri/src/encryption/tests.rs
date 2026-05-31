#[cfg(test)]
mod tests {
    use crate::encryption::aes::EncryptionManager;
    use sha2::{Sha256, Digest};

    #[test]
    fn test_content_hash_consistency() {
        let content = "Hello World";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_uniqueness() {
        let hash1 = hash_content("Hello World");
        let hash2 = hash_content("Hello World!");
        assert_ne!(hash1, hash2);
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
