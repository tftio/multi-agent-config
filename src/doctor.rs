//! Health check and diagnostics module.

use std::path::Path;
use workhelix_cli_common::DoctorCheck;

/// Run tool-specific health checks.
///
/// Returns a vector of health check results.
pub fn tool_specific_checks(config_path: &Path) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    // Check if config file exists
    if config_path.exists() {
        checks.push(DoctorCheck::pass(format!(
            "Config file found: {}",
            config_path.display()
        )));

        // Try to read and validate it
        match std::fs::read_to_string(config_path) {
            Ok(content) => {
                if toml::from_str::<toml::Value>(&content).is_ok() {
                    checks.push(DoctorCheck::pass("Config is valid TOML"));
                } else {
                    checks.push(DoctorCheck::fail(
                        "Config validation",
                        "Config is invalid TOML - run 'multi-agent-config validate' for details",
                    ));
                }
            }
            Err(e) => {
                checks.push(DoctorCheck::fail(
                    "Config read",
                    format!("Failed to read config: {e}"),
                ));
            }
        }
    } else {
        checks.push(DoctorCheck::fail(
            "Config file",
            format!(
                "Config not found at {} - run 'multi-agent-config init' to create it",
                config_path.display()
            ),
        ));
    }

    checks
}
