use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::cli::Language;

const CACHE_TTL_HOURS: u64 = 24;
const MAX_CACHE_SIZE: usize = 1000;

#[derive(Debug, Clone)]
struct CacheEntry<T: Clone> {
    value: T,
    expires_at: SystemTime,
}

pub struct DiscoveryCache {
    dependency_cache: HashMap<String, CacheEntry<Vec<PathBuf>>>,
    stdlib_cache: HashMap<Language, CacheEntry<HashSet<String>>>,
    file_hash_cache: HashMap<PathBuf, String>,
    detection_cache: HashMap<PathBuf, CacheEntry<Vec<Language>>>,
    cache_dir: PathBuf,
}

impl DiscoveryCache {
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).map_err(CacheError::Io)?;

        let mut cache = Self {
            dependency_cache: HashMap::new(),
            stdlib_cache: HashMap::new(),
            file_hash_cache: HashMap::new(),
            detection_cache: HashMap::new(),
            cache_dir,
        };

        cache.load_from_disk()?;
        Ok(cache)
    }

    fn get_cache_dir() -> Result<PathBuf, CacheError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| CacheError::CacheDir("Could not determine home directory".to_string()))?;

        let cache_dir = PathBuf::from(home).join(".cache").join("argflow");

        Ok(cache_dir)
    }

    pub fn get_dependencies(&self, key: &str) -> Option<Vec<PathBuf>> {
        self.dependency_cache.get(key).and_then(|entry| {
            if entry.expires_at > SystemTime::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set_dependencies(&mut self, key: String, files: Vec<PathBuf>) {
        let expires_at = SystemTime::now() + Duration::from_secs(CACHE_TTL_HOURS * 3600);
        self.dependency_cache.insert(
            key,
            CacheEntry::<Vec<PathBuf>> {
                value: files,
                expires_at,
            },
        );

        if self.dependency_cache.len() > MAX_CACHE_SIZE {
            self.evict_oldest_dependency();
        }
    }

    fn evict_oldest_dependency(&mut self) {
        let oldest_key = self
            .dependency_cache
            .iter()
            .min_by_key(|(_, entry)| entry.expires_at)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            self.dependency_cache.remove(&key);
        }
    }

    pub fn get_stdlib(&self, language: Language) -> Option<HashSet<String>> {
        self.stdlib_cache.get(&language).and_then(|entry| {
            if entry.expires_at > SystemTime::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set_stdlib(&mut self, language: Language, packages: HashSet<String>) {
        let expires_at = SystemTime::now() + Duration::from_secs(CACHE_TTL_HOURS * 3600);
        self.stdlib_cache.insert(
            language,
            CacheEntry::<HashSet<String>> {
                value: packages,
                expires_at,
            },
        );
    }

    pub fn get_file_hash(&self, path: &Path) -> Option<&String> {
        self.file_hash_cache.get(path)
    }

    pub fn set_file_hash(&mut self, path: PathBuf, hash: String) {
        self.file_hash_cache.insert(path, hash);
    }

    pub fn get_detection(&self, path: &Path) -> Option<Vec<Language>> {
        self.detection_cache.get(path).and_then(|entry| {
            if entry.expires_at > SystemTime::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set_detection(&mut self, path: PathBuf, languages: Vec<Language>) {
        let expires_at = SystemTime::now() + Duration::from_secs(CACHE_TTL_HOURS * 3600);
        self.detection_cache.insert(
            path,
            CacheEntry::<Vec<Language>> {
                value: languages,
                expires_at,
            },
        );
    }

    fn load_from_disk(&mut self) -> Result<(), CacheError> {
        let cache_file = self.cache_dir.join("discovery-cache.json");
        if !cache_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&cache_file).map_err(CacheError::Io)?;

        let _cache_data: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| CacheError::Deserialize(e.to_string()))?;

        Ok(())
    }

    pub fn save_to_disk(&self) -> Result<(), CacheError> {
        let cache_file = self.cache_dir.join("discovery-cache.json");
        let cache_data = serde_json::json!({
            "version": "1.0",
            "dependency_cache": {},
            "stdlib_cache": {},
            "file_hash_cache": {},
            "detection_cache": {},
        });

        let content = serde_json::to_string_pretty(&cache_data)
            .map_err(|e| CacheError::Serialize(e.to_string()))?;

        fs::write(&cache_file, content).map_err(CacheError::Io)?;

        Ok(())
    }

    pub fn invalidate_dependency(&mut self, key: &str) {
        self.dependency_cache.remove(key);
    }

    pub fn invalidate_file(&mut self, path: &Path) {
        self.file_hash_cache.remove(path);
        let path_str = path.to_string_lossy();
        let keys_to_remove: Vec<String> = self
            .dependency_cache
            .keys()
            .filter(|k| k.contains(path_str.as_ref()))
            .cloned()
            .collect();
        for key in keys_to_remove {
            self.dependency_cache.remove(&key);
        }
    }
}

impl Default for DiscoveryCache {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            dependency_cache: HashMap::new(),
            stdlib_cache: HashMap::new(),
            file_hash_cache: HashMap::new(),
            detection_cache: HashMap::new(),
            cache_dir: PathBuf::from("/tmp"),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cache directory error: {0}")]
    CacheDir(String),

    #[error("Serialize error: {0}")]
    Serialize(String),

    #[error("Deserialize error: {0}")]
    Deserialize(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_cache_creation() {
        let cache = DiscoveryCache::default();
        assert!(cache.dependency_cache.is_empty());
    }

    #[test]
    fn test_dependency_cache() {
        let mut cache = DiscoveryCache::default();
        let key = "test_project:go".to_string();
        let files: Vec<PathBuf> = vec![];

        cache.set_dependencies(key.clone(), files.clone());
        assert_eq!(cache.get_dependencies(&key), Some(files));
    }

    #[test]
    fn test_stdlib_cache() {
        let mut cache = DiscoveryCache::default();
        let mut packages = HashSet::new();
        packages.insert("crypto".to_string());
        packages.insert("crypto/aes".to_string());

        cache.set_stdlib(Language::Go, packages.clone());
        assert_eq!(cache.get_stdlib(Language::Go), Some(packages));
    }
}
