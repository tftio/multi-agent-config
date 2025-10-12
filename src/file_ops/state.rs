//! State tracking with SHA-256 hashing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// State file structure tracking generated configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    /// State file format version
    pub version: String,

    /// Last compilation timestamp
    pub last_compile: DateTime<Utc>,

    /// List of generated files
    pub generated_files: Vec<GeneratedFile>,
}

/// Record of a generated configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Tool name
    pub tool: String,

    /// File path
    pub path: PathBuf,

    /// Generation timestamp
    pub timestamp: DateTime<Utc>,

    /// SHA-256 hash with "sha256:" prefix
    pub hash: String,
}

/// State tracker for managing generated files
pub struct StateTracker {
    /// Path to state file
    state_file_path: PathBuf,

    /// Current state
    state: StateFile,
}

impl StateTracker {
    /// Load state from file, or create new if doesn't exist
    ///
    /// # Arguments
    ///
    /// * `path` - Path to state file
    ///
    /// # Returns
    ///
    /// * `Ok(StateTracker)` - Loaded or new state tracker
    /// * `Err(std::io::Error)` - Error loading state
    ///
    /// # Errors
    ///
    /// Returns error if state file exists but cannot be read
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let state = if path.exists() {
            let content = fs::read_to_string(path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| {
                eprintln!("Warning: Corrupted state file, creating new state");
                Self::new_state()
            })
        } else {
            Self::new_state()
        };

        Ok(Self {
            state_file_path: path.to_path_buf(),
            state,
        })
    }

    /// Create a new empty state
    fn new_state() -> StateFile {
        StateFile {
            version: "1.0".to_string(),
            last_compile: Utc::now(),
            generated_files: Vec::new(),
        }
    }

    /// Add a generated file to the state
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool name
    /// * `path` - File path
    /// * `hash` - SHA-256 hash
    pub fn add_generated_file(&mut self, tool: &str, path: PathBuf, hash: String) {
        // Remove existing entry for this path if present
        self.state.generated_files.retain(|f| f.path != path);

        // Add new entry
        self.state.generated_files.push(GeneratedFile {
            tool: tool.to_string(),
            path,
            timestamp: Utc::now(),
            hash,
        });

        // Update last compile timestamp
        self.state.last_compile = Utc::now();
    }

    /// Get the recorded hash for a file
    ///
    /// # Arguments
    ///
    /// * `path` - File path to look up
    ///
    /// # Returns
    ///
    /// Hash if found, None otherwise
    #[must_use]
    pub fn get_file_hash(&self, path: &Path) -> Option<String> {
        self.state
            .generated_files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.hash.clone())
    }

    /// Save state to file atomically
    ///
    /// # Returns
    ///
    /// * `Ok(())` - State saved successfully
    /// * `Err(std::io::Error)` - Error saving state
    ///
    /// # Errors
    ///
    /// Returns error if serialization or file write fails
    pub fn save(&self) -> Result<(), std::io::Error> {
        // Create parent directory if needed
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize state
        let json = serde_json::to_string_pretty(&self.state)?;

        // Write using atomic write
        crate::file_ops::writer::write_file_atomic(&self.state_file_path, &json, Some(0o600))
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(())
    }
}

#[allow(clippy::format_collect)]
/// Compute SHA-256 hash of a file
///
/// # Arguments
///
/// * `path` - File to hash
///
/// # Returns
///
/// * `Ok(String)` - Hash as "sha256:<hex>"
/// * `Err(std::io::Error)` - Error reading file
///
/// # Errors
///
/// Returns error if file cannot be read
pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("sha256:{hash:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_state_tracker_new() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let tracker = StateTracker::load(&state_path).unwrap();
        assert_eq!(tracker.state.version, "1.0");
        assert!(tracker.state.generated_files.is_empty());
    }

    #[test]
    fn test_state_tracker_add_file() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let mut tracker = StateTracker::load(&state_path).unwrap();
        tracker.add_generated_file(
            "cursor",
            PathBuf::from("/path/to/config.json"),
            "sha256:abc123".to_string(),
        );

        assert_eq!(tracker.state.generated_files.len(), 1);
        assert_eq!(tracker.state.generated_files[0].tool, "cursor");
        assert_eq!(tracker.state.generated_files[0].hash, "sha256:abc123");
    }

    #[test]
    fn test_state_tracker_get_hash() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let mut tracker = StateTracker::load(&state_path).unwrap();
        let path = PathBuf::from("/path/to/config.json");
        tracker.add_generated_file("cursor", path.clone(), "sha256:abc123".to_string());

        let hash = tracker.get_file_hash(&path);
        assert!(hash.is_some());
        assert_eq!(hash.unwrap(), "sha256:abc123");
    }

    #[test]
    fn test_state_tracker_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state").join("state.json");

        // Create and save state
        let mut tracker1 = StateTracker::load(&state_path).unwrap();
        tracker1.add_generated_file(
            "cursor",
            PathBuf::from("/test/config.json"),
            "sha256:test".to_string(),
        );
        tracker1.save().unwrap();

        // Load state in new tracker
        let tracker2 = StateTracker::load(&state_path).unwrap();
        assert_eq!(tracker2.state.generated_files.len(), 1);
        assert_eq!(tracker2.state.generated_files[0].tool, "cursor");
    }

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "test content").unwrap();

        let hash = hash_file(&file_path).unwrap();
        assert!(hash.starts_with("sha256:"));
        assert!(hash.len() > 10);
    }

    #[test]
    fn test_hash_file_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "same content").unwrap();

        let hash1 = hash_file(&file_path).unwrap();
        let hash2 = hash_file(&file_path).unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_file_different_content() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let hash1 = hash_file(&file1).unwrap();
        let hash2 = hash_file(&file2).unwrap();

        // Different content should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_state_tracker_replaces_existing_entry() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let mut tracker = StateTracker::load(&state_path).unwrap();
        let path = PathBuf::from("/test/config.json");

        // Add first entry
        tracker.add_generated_file("cursor", path.clone(), "sha256:old".to_string());
        assert_eq!(tracker.state.generated_files.len(), 1);

        // Add same path again with new hash
        tracker.add_generated_file("cursor", path, "sha256:new".to_string());
        assert_eq!(tracker.state.generated_files.len(), 1);
        assert_eq!(tracker.state.generated_files[0].hash, "sha256:new");
    }
}
