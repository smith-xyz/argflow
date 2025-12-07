/// Default crypto patterns for the PatternMatcher.
///
/// This is a simple MVP implementation. For production, use the classifier
/// module with the `crypto-classifier-rules` submodule for comprehensive
/// API mappings and algorithm classifications.
pub fn default_patterns() -> Vec<String> {
    vec![
        // Key derivation functions
        "pbkdf2".to_string(),
        "scrypt".to_string(),
        "argon2".to_string(),
        "bcrypt".to_string(),
        "hkdf".to_string(),
        // Symmetric encryption
        "aes".to_string(),
        "chacha".to_string(),
        "chacha20".to_string(),
        "des".to_string(),
        "3des".to_string(),
        "blowfish".to_string(),
        "twofish".to_string(),
        "rc4".to_string(),
        // Block cipher modes
        "gcm".to_string(),
        "cbc".to_string(),
        "ctr".to_string(),
        "cfb".to_string(),
        "ofb".to_string(),
        // Hashing
        "sha1".to_string(),
        "sha256".to_string(),
        "sha384".to_string(),
        "sha512".to_string(),
        "sha3".to_string(),
        "md5".to_string(),
        "md4".to_string(),
        "blake2".to_string(),
        "blake3".to_string(),
        "ripemd".to_string(),
        // Asymmetric / Public key
        "rsa".to_string(),
        "ecdsa".to_string(),
        "ecdh".to_string(),
        "ed25519".to_string(),
        "x25519".to_string(),
        "curve25519".to_string(),
        "dsa".to_string(),
        "dh".to_string(),
        // Message authentication
        "hmac".to_string(),
        "poly1305".to_string(),
        "cmac".to_string(),
        // Generic crypto operations
        "encrypt".to_string(),
        "decrypt".to_string(),
        "cipher".to_string(),
        "sign".to_string(),
        "verify".to_string(),
        "digest".to_string(),
        "hash".to_string(),
        "keyderive".to_string(),
        "keygen".to_string(),
        // Random number generation
        "rand".to_string(),
        "random".to_string(),
        "csprng".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_patterns_not_empty() {
        let patterns = default_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_contains_common_algorithms() {
        let patterns = default_patterns();
        assert!(patterns.contains(&"aes".to_string()));
        assert!(patterns.contains(&"sha256".to_string()));
        assert!(patterns.contains(&"pbkdf2".to_string()));
        assert!(patterns.contains(&"rsa".to_string()));
    }
}
