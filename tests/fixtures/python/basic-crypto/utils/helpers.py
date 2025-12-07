"""Utility functions - no crypto here."""


def format_key(key: bytes) -> str:
    """Format a key as hex string."""
    return key.hex()


def parse_key_size(size: str) -> int:
    """Parse key size from string."""
    size_map = {
        "128": 16,
        "256": 32,
    }
    return size_map.get(size.lower(), 32)


def validate_password(password: str) -> bool:
    """Check if password meets requirements."""
    return len(password) >= 8
