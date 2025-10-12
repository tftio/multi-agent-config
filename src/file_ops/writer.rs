//! Atomic file writing operations

use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Error type for file operations
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum FileOpError {
    /// I/O error during file operation
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to create parent directory
    #[error("Failed to create parent directory: {0}")]
    CreateDir(String),

    /// Failed to set file permissions
    #[error("Failed to set permissions: {0}")]
    Permission(String),

    /// Failed to persist temporary file
    #[error("Failed to persist file: {0}")]
    Persist(String),
}

/// Write content to a file atomically
///
/// This function uses a temporary file and atomic rename to ensure the write
/// is all-or-nothing. It creates parent directories if needed and sets
/// appropriate file permissions.
///
/// # Arguments
///
/// * `path` - Target file path
/// * `content` - Content to write
/// * `permissions` - Optional file permissions (Unix mode, e.g., 0o600). Defaults to 0o600 on Unix.
///
/// # Returns
///
/// * `Ok(())` - File written successfully
/// * `Err(FileOpError)` - Error during write operation
///
/// # Errors
///
/// Returns error if parent directory cannot be created, write fails, or permissions cannot be set
pub fn write_file_atomic(
    path: &Path,
    content: &str,
    permissions: Option<u32>,
) -> Result<(), FileOpError> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| FileOpError::CreateDir(format!("{}: {}", parent.display(), e)))?;
        }
    }

    // Create temporary file in the same directory as target
    let temp_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(temp_dir)?;

    // Write content to temporary file
    temp_file.write_all(content.as_bytes())?;

    // Flush to ensure all data is written
    temp_file.flush()?;

    // Set permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.unwrap_or(0o600);
        let perms = fs::Permissions::from_mode(mode);
        temp_file
            .as_file()
            .set_permissions(perms)
            .map_err(|e| FileOpError::Permission(e.to_string()))?;
    }

    // On Windows, permissions parameter is ignored (Windows has different permission model)
    #[cfg(windows)]
    {
        let _ = permissions; // Suppress unused warning
    }

    // Atomically persist the temporary file to the target path
    temp_file
        .persist(path)
        .map_err(|e| FileOpError::Persist(format!("{}: {}", path.display(), e.error)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_file_atomic_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = write_file_atomic(&file_path, "test content", None);
        assert!(result.is_ok());

        // Verify file exists and content is correct
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_file_atomic_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir
            .path()
            .join("subdir")
            .join("nested")
            .join("test.txt");

        let result = write_file_atomic(&file_path, "nested content", None);
        assert!(result.is_ok());

        // Verify file exists
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "nested content");
    }

    #[test]
    #[cfg(unix)]
    fn test_write_file_atomic_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = write_file_atomic(&file_path, "test", Some(0o600));
        assert!(result.is_ok());

        // Verify permissions
        let metadata = fs::metadata(&file_path).unwrap();
        let perms = metadata.permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }

    #[test]
    fn test_write_file_atomic_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Write initial content
        fs::write(&file_path, "original").unwrap();

        // Overwrite with atomic write
        let result = write_file_atomic(&file_path, "updated", None);
        assert!(result.is_ok());

        // Verify content updated
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "updated");
    }

    #[test]
    fn test_write_file_atomic_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");

        let result = write_file_atomic(&file_path, "", None);
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_write_file_atomic_large_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        // Create 1MB of content
        let large_content = "x".repeat(1024 * 1024);

        let result = write_file_atomic(&file_path, &large_content, None);
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.len(), 1024 * 1024);
    }
}
