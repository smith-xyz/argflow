"""SHA hashing functions."""

import hashlib


def hash_sha256(data: bytes) -> bytes:
    """Hash data with SHA-256."""
    h = hashlib.sha256()
    h.update(data)
    return h.digest()


def hash_sha512(data: bytes) -> bytes:
    """Hash data with SHA-512."""
    h = hashlib.sha512()
    h.update(data)
    return h.digest()


def get_hasher(algorithm: str):
    """Return a hasher based on algorithm name."""
    if algorithm == "sha256":
        return hashlib.sha256()
    elif algorithm == "sha512":
        return hashlib.sha512()
    else:
        return hashlib.sha256()
