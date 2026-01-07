use std::path::PathBuf;

// Allow dead_code: fixtures is included by multiple test crates (resolution_test,
// discover_test, scanner_test), and each only uses a subset of these functions.

#[allow(dead_code)]
pub fn get_test_fixture_path(language: &str, fixture_name: Option<&str>) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(language);

    if let Some(name) = fixture_name {
        path.push(name);
    }

    path
}

#[allow(dead_code)]
pub fn test_patterns() -> Vec<String> {
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
