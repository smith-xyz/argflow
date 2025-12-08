pub fn unquote_string(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('`') && s.ends_with('`'))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

pub fn extract_last_segment(path: &str) -> String {
    path.rsplit(['/', '.', ':'])
        .next()
        .unwrap_or(path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unquote_double_quotes() {
        assert_eq!(unquote_string("\"hello\""), "hello");
    }

    #[test]
    fn test_unquote_single_quotes() {
        assert_eq!(unquote_string("'hello'"), "hello");
    }

    #[test]
    fn test_unquote_backticks() {
        assert_eq!(unquote_string("`hello`"), "hello");
    }

    #[test]
    fn test_unquote_no_quotes() {
        assert_eq!(unquote_string("hello"), "hello");
    }

    #[test]
    fn test_extract_last_segment_slash() {
        assert_eq!(extract_last_segment("crypto/sha256"), "sha256");
        assert_eq!(extract_last_segment("golang.org/x/crypto/pbkdf2"), "pbkdf2");
    }

    #[test]
    fn test_extract_last_segment_double_colon() {
        assert_eq!(extract_last_segment("ring::pbkdf2"), "pbkdf2");
        assert_eq!(extract_last_segment("std::collections::HashMap"), "HashMap");
    }

    #[test]
    fn test_extract_last_segment_dot() {
        assert_eq!(extract_last_segment("hashlib.sha256"), "sha256");
    }

    #[test]
    fn test_extract_last_segment_simple() {
        assert_eq!(extract_last_segment("hashlib"), "hashlib");
    }
}
