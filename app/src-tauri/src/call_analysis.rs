//! Dev-only bridge to the local `pi` CLI: feeds it a journaled agent
//! call (request/response) and asks it to suggest project-code changes.
//! Debug builds only (`pi` is a personal dev tool, not a shipped-app
//! dependency); read-only tools are enforced so the call can only inspect
//! the codebase, never edit or execute anything in it.

use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const PI_READONLY_TOOLS: &str = "read,grep,find,ls";
const ANALYSIS_TIMEOUT_SECS: u64 = 180;

/// Args for `pi --print`, scoped to read-only tools so the analysis can
/// inspect the codebase but never edit/write/execute anything in it.
fn build_pi_args(model: &str, prompt: &str) -> Vec<String> {
    let mut args = Vec::new();
    if !model.is_empty() {
        args.push("--model".to_string());
        args.push(model.to_string());
    }
    args.push("--tools".to_string());
    args.push(PI_READONLY_TOOLS.to_string());
    args.push("--no-session".to_string());
    args.push("-p".to_string());
    args.push(prompt.to_string());
    args
}

/// The mlmail project root (two levels up from `src-tauri`), used as `pi`'s
/// working directory so it can read the actual project code.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .parent()
        .expect("app has a parent")
        .to_path_buf()
}

#[tauri::command]
pub async fn analyze_call_with_pi(prompt: String) -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME env var not set".to_string())?;
    let pi_path = PathBuf::from(home).join("bin").join("pi");
    if !pi_path.exists() {
        return Err(format!("pi binary not found at {}", pi_path.display()));
    }

    let model = std::env::var("N_CLOUD_MAX_MODEL").unwrap_or_default();
    let args = build_pi_args(&model, &prompt);

    let child = Command::new(&pi_path)
        .args(&args)
        .current_dir(project_root())
        .output();

    let output = timeout(Duration::from_secs(ANALYSIS_TIMEOUT_SECS), child)
        .await
        .map_err(|_| "pi analysis timed out".to_string())?
        .map_err(|e| format!("failed to run pi: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_pi_args_includes_model_and_readonly_tools() {
        let args = build_pi_args("openai-codex/gpt-5.5", "analyze this");
        assert_eq!(
            args,
            vec![
                "--model",
                "openai-codex/gpt-5.5",
                "--tools",
                "read,grep,find,ls",
                "--no-session",
                "-p",
                "analyze this",
            ]
        );
    }

    #[test]
    fn build_pi_args_omits_model_flag_when_empty() {
        let args = build_pi_args("", "analyze this");
        assert_eq!(
            args,
            vec![
                "--tools",
                "read,grep,find,ls",
                "--no-session",
                "-p",
                "analyze this"
            ]
        );
    }

    #[test]
    fn project_root_points_at_mlmail_checkout() {
        let root = project_root();
        assert!(root.join("app").join("src-tauri").is_dir());
    }
}
