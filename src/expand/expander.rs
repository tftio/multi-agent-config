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

    /// Expand shell environment variables (${VAR} syntax)
    ///
    /// Replaces all `${VAR}` patterns with values from shell environment.
    /// Undefined variables are replaced with empty string and generate a warning.
    ///
    /// # Arguments
    ///
    /// * `value` - String containing variable references
    ///
    /// # Returns
    ///
    /// Expanded string with all ${VAR} references resolved
    pub fn expand_shell_vars(&mut self, value: &str) -> String {
        use regex::Regex;

        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        let mut result = value.to_string();

        // Process all matches
        loop {
            let mut found_match = false;
            let result_clone = result.clone();

            for cap in re.captures_iter(&result_clone) {
                let full_match = cap.get(0).unwrap().as_str();
                let var_name = cap.get(1).unwrap().as_str();

                if let Some(var_value) = self.shell_env.get(var_name) {
                    result = result.replace(full_match, var_value);
                } else {
                    // Undefined variable - replace with empty string and warn
                    self.warnings
                        .push(format!("Shell variable '{}' is undefined", var_name));
                    result = result.replace(full_match, "");
                }
                found_match = true;
                break; // Restart from beginning after each replacement
            }

            if !found_match {
                break;
            }
        }

        result
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

    #[test]
    fn test_expand_shell_vars_single() {
        let mut shell_env = HashMap::new();
        shell_env.insert("HOME".to_string(), "/home/user".to_string());

        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_shell_vars("${HOME}/config");
        assert_eq!(result, "/home/user/config");
        assert!(expander.warnings().is_empty());
    }

    #[test]
    fn test_expand_shell_vars_multiple() {
        let mut shell_env = HashMap::new();
        shell_env.insert("USER".to_string(), "alice".to_string());
        shell_env.insert("HOST".to_string(), "server".to_string());

        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_shell_vars("${USER}@${HOST}");
        assert_eq!(result, "alice@server");
        assert!(expander.warnings().is_empty());
    }

    #[test]
    fn test_expand_shell_vars_undefined() {
        let shell_env = HashMap::new();
        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_shell_vars("${UNDEFINED}");
        assert_eq!(result, "");
        assert_eq!(expander.warnings().len(), 1);
        assert!(expander.warnings()[0].contains("UNDEFINED"));
        assert!(expander.warnings()[0].contains("undefined"));
    }

    #[test]
    fn test_expand_shell_vars_mixed_defined_undefined() {
        let mut shell_env = HashMap::new();
        shell_env.insert("DEFINED".to_string(), "value".to_string());

        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_shell_vars("${DEFINED}-${UNDEFINED}");
        assert_eq!(result, "value-");
        assert_eq!(expander.warnings().len(), 1);
    }

    #[test]
    fn test_expand_shell_vars_no_substitution() {
        let shell_env = HashMap::new();
        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_shell_vars("plain text");
        assert_eq!(result, "plain text");
        assert!(expander.warnings().is_empty());
    }

    #[test]
    fn test_expand_shell_vars_nested_braces() {
        let mut shell_env = HashMap::new();
        shell_env.insert("VAR".to_string(), "value".to_string());

        let env_section = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        // {VAR} should not be expanded by shell expander
        let result = expander.expand_shell_vars("{VAR} and ${VAR}");
        assert_eq!(result, "{VAR} and value");
    }
}
