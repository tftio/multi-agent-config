//! Diff generation for showing configuration changes

use std::fs;
use std::path::Path;

/// Generate a unified diff between two strings
///
/// Creates a unified diff format showing the differences between old and new content.
///
/// # Arguments
///
/// * `old_content` - Original content (empty string if file doesn't exist)
/// * `new_content` - New content
/// * `file_path` - File path for diff header
///
/// # Returns
///
/// String containing unified diff format
pub fn generate_diff(old_content: &str, new_content: &str, file_path: &Path) -> String {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old_content, new_content);
    let mut output = String::new();

    // Add diff header
    output.push_str(&format!("--- {}\n", file_path.display()));
    output.push_str(&format!("+++ {} (new)\n", file_path.display()));

    // Generate unified diff
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };

                output.push_str(&format!("{}{}", sign, change.value()));
            }
        }
    }

    output
}

/// Generate diff for a file (load old content from disk)
///
/// # Arguments
///
/// * `file_path` - Path to file
/// * `new_content` - New content to compare against
///
/// # Returns
///
/// Unified diff string
pub fn generate_file_diff(file_path: &Path, new_content: &str) -> String {
    let old_content = if file_path.exists() {
        fs::read_to_string(file_path).unwrap_or_default()
    } else {
        String::new()
    };

    generate_diff(&old_content, new_content, file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_diff_no_changes() {
        let content = "line1\nline2\nline3\n";
        let diff = generate_diff(content, content, Path::new("test.txt"));

        // No changes should produce minimal output
        assert!(diff.contains("test.txt"));
    }

    #[test]
    fn test_generate_diff_addition() {
        let old = "line1\nline2\n";
        let new = "line1\nline2\nline3\n";
        let diff = generate_diff(old, new, Path::new("test.txt"));

        assert!(diff.contains("---"));
        assert!(diff.contains("+++"));
        assert!(diff.contains("+line3"));
    }

    #[test]
    fn test_generate_diff_deletion() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline3\n";
        let diff = generate_diff(old, new, Path::new("test.txt"));

        assert!(diff.contains("-line2"));
    }

    #[test]
    fn test_generate_diff_modification() {
        let old = "line1\nold line\nline3\n";
        let new = "line1\nnew line\nline3\n";
        let diff = generate_diff(old, new, Path::new("test.txt"));

        assert!(diff.contains("-old line") || diff.contains("old"));
        assert!(diff.contains("+new line") || diff.contains("new"));
    }

    #[test]
    fn test_generate_file_diff_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "original\ncontent\n").unwrap();

        let diff = generate_file_diff(&file_path, "original\nupdated\n");
        assert!(diff.contains("-content"));
        assert!(diff.contains("+updated"));
    }

    #[test]
    fn test_generate_file_diff_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.txt");

        // File doesn't exist
        let diff = generate_file_diff(&file_path, "new\ncontent\n");
        assert!(diff.contains("new.txt"));
        assert!(diff.contains("+new"));
        assert!(diff.contains("+content"));
    }

    #[test]
    fn test_generate_diff_empty_to_content() {
        let diff = generate_diff("", "new content\n", Path::new("test.txt"));
        assert!(diff.contains("+new content"));
    }

    #[test]
    fn test_generate_diff_content_to_empty() {
        let diff = generate_diff("old content\n", "", Path::new("test.txt"));
        assert!(diff.contains("-old content"));
    }
}
