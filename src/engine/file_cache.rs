use std::collections::HashMap;
use std::path::Path;

const MAX_FILE_CACHE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct CachedFileEntry {
    pub constants: HashMap<String, crate::Value>,
    pub functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub file_path: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Default)]
pub struct FileCache {
    entries: HashMap<String, CachedFileEntry>,
    load_order: Vec<String>,
}

impl FileCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, file_path: String, entry: CachedFileEntry) {
        if self.entries.len() >= MAX_FILE_CACHE_SIZE && !self.entries.contains_key(&file_path) {
            if let Some(oldest) = self.load_order.first().cloned() {
                self.entries.remove(&oldest);
                self.load_order.remove(0);
            }
        }

        if !self.entries.contains_key(&file_path) {
            self.load_order.push(file_path.clone());
        }
        self.entries.insert(file_path, entry);
    }

    pub fn get_file(&self, file_path: &str) -> Option<&CachedFileEntry> {
        self.entries.get(file_path)
    }

    pub fn find_constant(&self, name: &str) -> Option<crate::Value> {
        for entry in self.entries.values() {
            if let Some(value) = entry.constants.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn find_constant_in_package(&self, name: &str, package_dir: &str) -> Option<crate::Value> {
        for (path, entry) in &self.entries {
            if let Some(parent) = Path::new(path).parent() {
                if parent.to_string_lossy() == package_dir {
                    if let Some(value) = entry.constants.get(name) {
                        return Some(value.clone());
                    }
                }
            }
        }
        None
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionInfo> {
        for entry in self.entries.values() {
            if let Some(info) = entry.functions.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.load_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_cache_add_and_find_constant() {
        let mut cache = FileCache::new();

        let mut constants = HashMap::new();
        constants.insert("MAX_ITER".to_string(), crate::Value::resolved_int(100000));

        cache.add_file(
            "/pkg/constants.go".to_string(),
            CachedFileEntry {
                constants,
                functions: HashMap::new(),
            },
        );

        let found = cache.find_constant("MAX_ITER");
        assert!(found.is_some());
        assert!(found.unwrap().is_resolved);
    }

    #[test]
    fn test_file_cache_find_constant_in_package() {
        let mut cache = FileCache::new();

        let mut constants1 = HashMap::new();
        constants1.insert("CONST_A".to_string(), crate::Value::resolved_int(100));

        let mut constants2 = HashMap::new();
        constants2.insert("CONST_B".to_string(), crate::Value::resolved_int(200));

        cache.add_file(
            "/pkg/a.go".to_string(),
            CachedFileEntry {
                constants: constants1,
                functions: HashMap::new(),
            },
        );

        cache.add_file(
            "/other/b.go".to_string(),
            CachedFileEntry {
                constants: constants2,
                functions: HashMap::new(),
            },
        );

        let found = cache.find_constant_in_package("CONST_A", "/pkg");
        assert!(found.is_some());

        let not_found = cache.find_constant_in_package("CONST_B", "/pkg");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_file_cache_find_function() {
        let mut cache = FileCache::new();

        let mut functions = HashMap::new();
        functions.insert(
            "getIterations".to_string(),
            FunctionInfo {
                file_path: "/pkg/utils.go".to_string(),
                start_byte: 100,
                end_byte: 200,
            },
        );

        cache.add_file(
            "/pkg/utils.go".to_string(),
            CachedFileEntry {
                constants: HashMap::new(),
                functions,
            },
        );

        let found = cache.find_function("getIterations");
        assert!(found.is_some());
        let info = found.unwrap();
        assert_eq!(info.start_byte, 100);
        assert_eq!(info.end_byte, 200);
    }

    #[test]
    fn test_file_cache_lru_eviction() {
        let mut cache = FileCache::new();

        for i in 0..MAX_FILE_CACHE_SIZE + 5 {
            let mut constants = HashMap::new();
            constants.insert(format!("CONST_{}", i), crate::Value::resolved_int(i as i64));

            cache.add_file(
                format!("/pkg/file_{}.go", i),
                CachedFileEntry {
                    constants,
                    functions: HashMap::new(),
                },
            );
        }

        assert_eq!(cache.file_count(), MAX_FILE_CACHE_SIZE);

        assert!(cache.find_constant("CONST_0").is_none());
        assert!(cache
            .find_constant(&format!("CONST_{}", MAX_FILE_CACHE_SIZE + 4))
            .is_some());
    }

    #[test]
    fn test_file_cache_clear() {
        let mut cache = FileCache::new();

        let mut constants = HashMap::new();
        constants.insert("CONST".to_string(), crate::Value::resolved_int(1));
        cache.add_file(
            "/pkg/a.go".to_string(),
            CachedFileEntry {
                constants,
                functions: HashMap::new(),
            },
        );

        assert_eq!(cache.file_count(), 1);

        cache.clear();

        assert_eq!(cache.file_count(), 0);
        assert!(cache.find_constant("CONST").is_none());
    }
}
