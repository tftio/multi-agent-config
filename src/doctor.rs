//! Health check and diagnostics module.

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
pub fn run_doctor() -> i32 {
    println!("🏥 multi-agent-config health check");
    println!("========================");
    println!();

    let mut has_warnings = false;

    // Check for updates
    println!("Updates:");
    match check_for_updates() {
        Ok(Some(latest)) => {
            let current = env!("CARGO_PKG_VERSION");
            println!("  ⚠️  Update available: v{latest} (current: v{current})");
            println!("  💡 Run 'multi-agent-config update' to install the latest version");
            has_warnings = true;
        }
        Ok(None) => {
            println!("  ✅ Running latest version (v{})", env!("CARGO_PKG_VERSION"));
        }
        Err(e) => {
            println!("  ⚠️  Failed to check for updates: {e}");
            has_warnings = true;
        }
    }

    println!();

    // Summary
    if has_warnings {
        println!("⚠️  Warnings found");
        0 // Warnings don't cause failure
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}

fn check_for_updates() -> Result<Option<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("multi-agent-config-doctor")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let repo_url = "https://github.com/jfb/multi-agent-config";
    let repo_path = repo_url
        .trim_end_matches(".git")
        .trim_end_matches('/')
        .split('/')
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/");

    let url = format!("https://api.github.com/repos/{}/releases/latest", repo_path);
    let response: serde_json::Value = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let tag_name = response["tag_name"]
        .as_str()
        .ok_or_else(|| "No tag_name in response".to_string())?;

    let latest = tag_name
        .trim_start_matches("multi-agent-config-v")
        .trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");

    if latest != current {
        Ok(Some(latest.to_string()))
    } else {
        Ok(None)
    }
}
