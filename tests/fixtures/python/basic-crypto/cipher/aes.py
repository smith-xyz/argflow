"""AES encryption functions."""

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes  # pyright: ignore[reportMissingImports]
from cryptography.hazmat.backends import default_backend  # pyright: ignore[reportMissingImports]

KEY_SIZE_128 = 16
KEY_SIZE_256 = 32


def create_aes_128_cipher(key: bytes, iv: bytes):
    """Create an AES-128 cipher in CBC mode."""
    cipher = Cipher(algorithms.AES(key[:16]), modes.CBC(iv), backend=default_backend())
    return cipher.encryptor()


def create_aes_256_cipher(key: bytes, iv: bytes):
    """Create an AES-256 cipher with literal key size."""
    cipher = Cipher(algorithms.AES(key[:32]), modes.CBC(iv), backend=default_backend())
    return cipher.encryptor()


def create_aes_gcm(key: bytes, nonce: bytes):
    """Create an AES-GCM cipher."""
    cipher = Cipher(algorithms.AES(key), modes.GCM(nonce), backend=default_backend())
    return cipher.encryptor()
