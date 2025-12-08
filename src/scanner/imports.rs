use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ImportMap {
    imports: HashMap<String, String>,
}

impl ImportMap {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    pub fn insert(&mut self, short_name: String, full_path: String) {
        self.imports.insert(short_name, full_path);
    }

    pub fn get(&self, short_name: &str) -> Option<&String> {
        self.imports.get(short_name)
    }

    pub fn resolve(&self, package: &str) -> Option<String> {
        self.imports.get(package).cloned()
    }

    pub fn len(&self) -> usize {
        self.imports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.imports.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_map_basic() {
        let mut imports = ImportMap::new();
        imports.insert("sha256".to_string(), "crypto/sha256".to_string());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("sha256"), Some(&"crypto/sha256".to_string()));
        assert_eq!(imports.resolve("sha256"), Some("crypto/sha256".to_string()));
    }

    #[test]
    fn test_import_map_not_found() {
        let imports = ImportMap::new();
        assert_eq!(imports.get("nonexistent"), None);
        assert_eq!(imports.resolve("nonexistent"), None);
    }
}
