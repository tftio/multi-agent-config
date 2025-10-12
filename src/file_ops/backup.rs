//! Backup creation and restoration

use std::{
    fs,
    path::{Path, PathBuf},
};

#[allow(clippy::format_collect)]
/// Create a backup of an existing file
///
/// Creates a backup with `.backup` extension. If the original file doesn't
/// exist, no backup is created and Ok(None) is returned.
///
/// # Arguments
///
/// * `original_path` - Path to the file to backup
///
/// # Returns
///
/// * `Ok(Some(PathBuf))` - Backup created, returns backup path
/// * `Ok(None)` - Original file doesn't exist, no backup needed
/// * `Err(std::io::Error)` - Error creating backup
///
/// # Errors
///
/// Returns error if file cannot be copied
pub fn create_backup(original_path: &Path) -> Result<Option<PathBuf>, std::io::Error> {
    // Check if original exists
    if !original_path.exists() {
        return Ok(None);
    }

    // Generate backup path
    let backup_path = original_path.with_extension(
        format!(
            "{}.backup",
            original_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
        )
        .trim_start_matches('.'),
    );

    // Copy file to backup
    fs::copy(original_path, &backup_path)?;

    Ok(Some(backup_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_backup_success() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("config.toml");

        // Create original file
        fs::write(&original, "original content").unwrap();

        let result = create_backup(&original);
        assert!(result.is_ok());

        let backup_path = result.unwrap();
        assert!(backup_path.is_some());

        let backup = backup_path.unwrap();
        assert!(backup.exists());
        assert!(backup.to_string_lossy().contains("backup"));

        // Verify backup content
        let content = fs::read_to_string(&backup).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_create_backup_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("nonexistent.toml");

        let result = create_backup(&original);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_create_backup_overwrites_existing_backup() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("config.toml");

        // Create original file
        fs::write(&original, "first version").unwrap();

        // Create first backup
        let result1 = create_backup(&original);
        assert!(result1.is_ok());
        let backup1 = result1.unwrap().unwrap();

        // Modify original
        fs::write(&original, "second version").unwrap();

        // Create second backup (should overwrite first)
        let result2 = create_backup(&original);
        assert!(result2.is_ok());
        let backup2 = result2.unwrap().unwrap();

        // Backup paths should be the same
        assert_eq!(backup1, backup2);

        // Backup should contain second version
        let content = fs::read_to_string(&backup2).unwrap();
        assert_eq!(content, "second version");
    }

    #[test]
    fn test_create_backup_preserves_content() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("data.json");

        let original_content = r#"{"key": "value", "nested": {"data": [1, 2, 3]}}"#;
        fs::write(&original, original_content).unwrap();

        let result = create_backup(&original);
        assert!(result.is_ok());

        let backup = result.unwrap().unwrap();
        let backup_content = fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_content, original_content);
    }
}
