"""PBKDF2 key derivation functions."""

import hashlib

DEFAULT_ITERATIONS = 100000
DEFAULT_KEY_LENGTH = 32


def derive_key(password: bytes, salt: bytes) -> bytes:
    """Derive a key using PBKDF2 with default parameters."""
    return hashlib.pbkdf2_hmac(
        "sha256", password, salt, DEFAULT_ITERATIONS, DEFAULT_KEY_LENGTH
    )


def derive_key_custom(password: bytes, salt: bytes, iterations: int) -> bytes:
    """Derive a key with custom iterations."""
    return hashlib.pbkdf2_hmac("sha256", password, salt, iterations, 32)


def derive_key_literal(password: bytes, salt: bytes) -> bytes:
    """Derive a key with literal values."""
    return hashlib.pbkdf2_hmac("sha256", password, salt, 10000, 32)
