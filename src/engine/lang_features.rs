//! Language-specific parsing features and quirks.
//!
//! This module centralizes knowledge about language-specific lexical features
//! that affect how literals are parsed, such as:
//! - Integer type suffixes (Rust: u32, i64, etc.)
//! - Number format prefixes (0x, 0o, 0b)
//! - String prefixes (Python: r"", b"", f"")

use super::Language;

pub const RUST_INT_SUFFIXES: &[&str] = &[
    "u128", "i128", "usize", "isize", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];

pub fn strip_int_suffix(text: &str, language: Language) -> &str {
    match language {
        Language::Rust => strip_rust_int_suffix(text),
        _ => text,
    }
}

fn strip_rust_int_suffix(text: &str) -> &str {
    for suffix in RUST_INT_SUFFIXES {
        if let Some(stripped) = text.strip_suffix(suffix) {
            return stripped;
        }
    }
    text
}

pub fn parse_int_literal(text: &str, language: Language) -> Option<i64> {
    let text = text.trim().replace('_', "");
    let text = strip_int_suffix(&text, language);

    if text.starts_with("0x") || text.starts_with("0X") {
        i64::from_str_radix(&text[2..], 16).ok()
    } else if text.starts_with("0o") || text.starts_with("0O") {
        i64::from_str_radix(&text[2..], 8).ok()
    } else if text.starts_with("0b") || text.starts_with("0B") {
        i64::from_str_radix(&text[2..], 2).ok()
    } else if text.starts_with('0') && text.len() > 1 && !text.contains('.') {
        // Go-style octal (0755) - try octal first, fall back to decimal
        i64::from_str_radix(&text[1..], 8)
            .ok()
            .or_else(|| text.parse().ok())
    } else {
        text.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_rust_int_suffix() {
        assert_eq!(strip_rust_int_suffix("100u32"), "100");
        assert_eq!(strip_rust_int_suffix("200i64"), "200");
        assert_eq!(strip_rust_int_suffix("300usize"), "300");
        assert_eq!(strip_rust_int_suffix("400"), "400");
    }

    #[test]
    fn test_parse_int_decimal() {
        assert_eq!(parse_int_literal("100", Language::Go), Some(100));
        assert_eq!(parse_int_literal("100_000", Language::Go), Some(100000));
    }

    #[test]
    fn test_parse_int_hex() {
        assert_eq!(parse_int_literal("0xFF", Language::Go), Some(255));
        assert_eq!(parse_int_literal("0XFF", Language::Python), Some(255));
    }

    #[test]
    fn test_parse_int_octal() {
        assert_eq!(parse_int_literal("0o40", Language::Python), Some(32));
        assert_eq!(parse_int_literal("0O40", Language::Python), Some(32));
    }

    #[test]
    fn test_parse_int_binary() {
        assert_eq!(parse_int_literal("0b100000", Language::Rust), Some(32));
        assert_eq!(parse_int_literal("0B100000", Language::Rust), Some(32));
    }

    #[test]
    fn test_parse_int_go_octal() {
        // Go-style octal without 'o'
        assert_eq!(parse_int_literal("0755", Language::Go), Some(493));
    }

    #[test]
    fn test_parse_int_rust_suffixes() {
        assert_eq!(parse_int_literal("100u32", Language::Rust), Some(100));
        assert_eq!(parse_int_literal("200i64", Language::Rust), Some(200));
        assert_eq!(parse_int_literal("300usize", Language::Rust), Some(300));
        assert_eq!(parse_int_literal("0xFFu8", Language::Rust), Some(255));
    }

    #[test]
    fn test_parse_int_non_rust_ignores_suffix() {
        // Other languages don't strip suffixes (they'd be syntax errors anyway)
        assert_eq!(parse_int_literal("100u32", Language::Go), None);
    }
}
