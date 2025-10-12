//! Variable expansion implementation

use std::collections::HashMap;

/// Maximum depth for variable expansion to prevent infinite loops
pub const MAX_EXPANSION_DEPTH: usize = 10;

/// Result type for expansion operations
pub type ExpansionResult = Result<String, ExpansionError>;

/// Errors that can occur during variable expansion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpansionError {
    /// Circular reference detected
    CircularReference {
        /// Variable name that caused the circular reference
        var_name: String,
        /// Depth at which circular reference was detected
        depth: usize,
    },
    /// Maximum expansion depth exceeded
    MaxDepthExceeded {
        /// Current depth when limit was hit
        current_depth: usize,
        /// Maximum allowed depth
        max_depth: usize,
    },
}

impl std::fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpansionError::CircularReference { var_name, depth } => {
                write!(
                    f,
                    "Circular reference detected for variable '{}' at depth {}",
                    var_name, depth
                )
            }
            ExpansionError::MaxDepthExceeded {
                current_depth,
                max_depth,
            } => {
                write!(
                    f,
                    "Maximum expansion depth exceeded: {} > {}",
                    current_depth, max_depth
                )
            }
        }
    }
}

impl std::error::Error for ExpansionError {}

/// Variable expander that handles both shell environment and config [env] section
pub struct Expander {
    /// Variables from [env] section of config
    env_section: HashMap<String, String>,
    /// Shell environment variables
    shell_env: HashMap<String, String>,
    /// Warnings collected during expansion
    warnings: Vec<String>,
}

impl Expander {
    /// Create a new expander
    ///
    /// # Arguments
    ///
    /// * `env_section` - Variables from config [env] section
    /// * `shell_env` - Shell environment variables
    pub fn new(env_section: HashMap<String, String>, shell_env: HashMap<String, String>) -> Self {
        Self {
            env_section,
            shell_env,
            warnings: Vec::new(),
        }
    }

    /// Get collected warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Clear collected warnings
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_new() {
        let env_section = HashMap::new();
        let shell_env = HashMap::new();
        let expander = Expander::new(env_section, shell_env);
        assert!(expander.warnings().is_empty());
    }

    #[test]
    fn test_expansion_error_display_circular() {
        let err = ExpansionError::CircularReference {
            var_name: "FOO".to_string(),
            depth: 5,
        };
        assert!(format!("{err}").contains("Circular reference"));
        assert!(format!("{err}").contains("FOO"));
        assert!(format!("{err}").contains("5"));
    }

    #[test]
    fn test_expansion_error_display_max_depth() {
        let err = ExpansionError::MaxDepthExceeded {
            current_depth: 11,
            max_depth: 10,
        };
        assert!(format!("{err}").contains("Maximum expansion depth"));
        assert!(format!("{err}").contains("11"));
        assert!(format!("{err}").contains("10"));
    }

    #[test]
    fn test_max_expansion_depth_constant() {
        assert_eq!(MAX_EXPANSION_DEPTH, 10);
    }
}
