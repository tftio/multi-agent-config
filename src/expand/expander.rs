//! Variable expansion implementation

use std::collections::{HashMap, HashSet};

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

    /// Expand config environment variables ({VAR} syntax) with circular reference detection
    ///
    /// Replaces all `{VAR}` patterns with values from [env] section.
    /// Recursively expands nested references with depth tracking.
    /// Undefined variables are replaced with empty string and generate a warning.
    ///
    /// # Arguments
    ///
    /// * `value` - String containing variable references
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Expanded string with all {VAR} references resolved
    /// * `Err(ExpansionError)` - Circular reference or max depth exceeded
    ///
    /// # Errors
    ///
    /// Returns error if circular reference detected or max depth exceeded
    pub fn expand_env_vars(&mut self, value: &str) -> ExpansionResult {
        let mut visited = HashSet::new();
        self.expand_env_vars_recursive(value, 0, &mut visited)
    }

    /// Recursively expand env variables with depth and circular reference tracking
    fn expand_env_vars_recursive(
        &mut self,
        value: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> ExpansionResult {
        // Check depth limit
        if depth >= MAX_EXPANSION_DEPTH {
            return Err(ExpansionError::MaxDepthExceeded {
                current_depth: depth,
                max_depth: MAX_EXPANSION_DEPTH,
            });
        }

        use regex::Regex;
        let re = Regex::new(r"\{([^}]+)\}").unwrap();
        let mut result = value.to_string();

        // Process all matches
        loop {
            let mut found_match = false;
            let result_clone = result.clone();

            for cap in re.captures_iter(&result_clone) {
                let full_match = cap.get(0).unwrap().as_str();
                let var_name = cap.get(1).unwrap().as_str();

                // Check for circular reference
                if visited.contains(var_name) {
                    return Err(ExpansionError::CircularReference {
                        var_name: var_name.to_string(),
                        depth,
                    });
                }

                if let Some(var_value) = self.env_section.get(var_name).cloned() {
                    // Mark as visited
                    visited.insert(var_name.to_string());

                    // First expand any shell variables in this value
                    let after_shell = self.expand_shell_vars(&var_value);

                    // Then recursively expand config variables
                    let expanded = self.expand_env_vars_recursive(&after_shell, depth + 1, visited)?;

                    // Unmark after expansion
                    visited.remove(var_name);

                    result = result.replace(full_match, &expanded);
                } else {
                    // Undefined variable - replace with empty string and warn
                    self.warnings
                        .push(format!("Config variable '{}' is undefined", var_name));
                    result = result.replace(full_match, "");
                }
                found_match = true;
                break; // Restart from beginning after each replacement
            }

            if !found_match {
                break;
            }
        }

        Ok(result)
    }

    /// Expand all variables in a string (both shell ${VAR} and config {VAR})
    ///
    /// First expands shell variables, then config variables.
    ///
    /// # Arguments
    ///
    /// * `value` - String containing variable references
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Fully expanded string
    /// * `Err(ExpansionError)` - Expansion error (circular reference or max depth)
    ///
    /// # Errors
    ///
    /// Returns error if circular reference detected or max depth exceeded
    pub fn expand(&mut self, value: &str) -> ExpansionResult {
        // First expand shell variables ${VAR}
        let after_shell = self.expand_shell_vars(value);

        // Then expand config variables {VAR}
        self.expand_env_vars(&after_shell)
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

    #[test]
    fn test_expand_env_vars_single() {
        let mut env_section = HashMap::new();
        env_section.insert("API_KEY".to_string(), "secret123".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("key={API_KEY}").unwrap();
        assert_eq!(result, "key=secret123");
        assert!(expander.warnings().is_empty());
    }

    #[test]
    fn test_expand_env_vars_multiple() {
        let mut env_section = HashMap::new();
        env_section.insert("HOST".to_string(), "localhost".to_string());
        env_section.insert("PORT".to_string(), "8080".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{HOST}:{PORT}").unwrap();
        assert_eq!(result, "localhost:8080");
    }

    #[test]
    fn test_expand_env_vars_nested() {
        let mut env_section = HashMap::new();
        env_section.insert("A".to_string(), "value_a".to_string());
        env_section.insert("B".to_string(), "{A}_b".to_string());
        env_section.insert("C".to_string(), "{B}_c".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{C}").unwrap();
        assert_eq!(result, "value_a_b_c");
    }

    #[test]
    fn test_expand_env_vars_undefined() {
        let env_section = HashMap::new();
        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{UNDEFINED}").unwrap();
        assert_eq!(result, "");
        assert_eq!(expander.warnings().len(), 1);
        assert!(expander.warnings()[0].contains("UNDEFINED"));
    }

    #[test]
    fn test_expand_env_vars_circular_simple() {
        let mut env_section = HashMap::new();
        env_section.insert("A".to_string(), "{B}".to_string());
        env_section.insert("B".to_string(), "{A}".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{A}");
        assert!(result.is_err());
        match result {
            Err(ExpansionError::CircularReference { var_name, .. }) => {
                assert!(var_name == "A" || var_name == "B");
            }
            _ => panic!("Expected CircularReference error"),
        }
    }

    #[test]
    fn test_expand_env_vars_circular_complex() {
        let mut env_section = HashMap::new();
        env_section.insert("A".to_string(), "{B}".to_string());
        env_section.insert("B".to_string(), "{C}".to_string());
        env_section.insert("C".to_string(), "{A}".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{A}");
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_env_vars_self_reference() {
        let mut env_section = HashMap::new();
        env_section.insert("A".to_string(), "{A}".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{A}");
        assert!(result.is_err());
        match result {
            Err(ExpansionError::CircularReference { var_name, depth }) => {
                assert_eq!(var_name, "A");
                assert_eq!(depth, 1); // Depth is 1 when we detect the cycle
            }
            _ => panic!("Expected CircularReference error"),
        }
    }

    #[test]
    fn test_expand_env_vars_max_depth() {
        let mut env_section = HashMap::new();
        // Create deep nesting: A -> B -> C -> ... -> K (11 levels)
        env_section.insert("A".to_string(), "{B}".to_string());
        env_section.insert("B".to_string(), "{C}".to_string());
        env_section.insert("C".to_string(), "{D}".to_string());
        env_section.insert("D".to_string(), "{E}".to_string());
        env_section.insert("E".to_string(), "{F}".to_string());
        env_section.insert("F".to_string(), "{G}".to_string());
        env_section.insert("G".to_string(), "{H}".to_string());
        env_section.insert("H".to_string(), "{I}".to_string());
        env_section.insert("I".to_string(), "{J}".to_string());
        env_section.insert("J".to_string(), "{K}".to_string());
        env_section.insert("K".to_string(), "value".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{A}");
        assert!(result.is_err());
        match result {
            Err(ExpansionError::MaxDepthExceeded { current_depth, .. }) => {
                assert_eq!(current_depth, MAX_EXPANSION_DEPTH);
            }
            _ => panic!("Expected MaxDepthExceeded error"),
        }
    }

    #[test]
    fn test_expand_env_vars_deep_but_valid() {
        let mut env_section = HashMap::new();
        // Create deep nesting: A -> B -> ... -> I (9 levels, just under limit)
        env_section.insert("A".to_string(), "{B}".to_string());
        env_section.insert("B".to_string(), "{C}".to_string());
        env_section.insert("C".to_string(), "{D}".to_string());
        env_section.insert("D".to_string(), "{E}".to_string());
        env_section.insert("E".to_string(), "{F}".to_string());
        env_section.insert("F".to_string(), "{G}".to_string());
        env_section.insert("G".to_string(), "{H}".to_string());
        env_section.insert("H".to_string(), "{I}".to_string());
        env_section.insert("I".to_string(), "value".to_string());

        let shell_env = HashMap::new();
        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand_env_vars("{A}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "value");
    }

    #[test]
    fn test_expand_combined_shell_and_env() {
        let mut shell_env = HashMap::new();
        shell_env.insert("HOME".to_string(), "/home/user".to_string());

        let mut env_section = HashMap::new();
        env_section.insert("CONFIG_DIR".to_string(), "${HOME}/.config".to_string());

        let mut expander = Expander::new(env_section, shell_env);

        let result = expander.expand("{CONFIG_DIR}/app").unwrap();
        assert_eq!(result, "/home/user/.config/app");
    }

    #[test]
    fn test_expand_shell_first_then_env() {
        let mut shell_env = HashMap::new();
        shell_env.insert("BASE".to_string(), "/opt".to_string());

        let mut env_section = HashMap::new();
        env_section.insert("PATH_PREFIX".to_string(), "${BASE}/bin".to_string());

        let mut expander = Expander::new(env_section, shell_env);

        // ${BASE} expanded first by shell, then {PATH_PREFIX} by env
        let result = expander.expand("exe={PATH_PREFIX}/app").unwrap();
        assert_eq!(result, "exe=/opt/bin/app");
    }
}
