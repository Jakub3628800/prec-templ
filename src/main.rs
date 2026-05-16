//! prec-templ: Auto-detect technologies and generate pre-commit configuration files.
//!
//! This is the main entry point for the CLI tool.

mod cli;
mod config;
mod discover;
mod render;
mod ui;

use cli::Cli;
use console::style;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn main() {
    if let Err(e) = run() {
        ui::print_error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Cli::parse_args();
    let path = resolve_directory_path(&args.path)?;

    if args.interactive {
        run_interactive(&path)
    } else {
        run_auto(&path, args.install)
    }
}

fn resolve_directory_path(path: &Path) -> Result<std::path::PathBuf, String> {
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;
    if !resolved.is_dir() {
        return Err(format!("Path is not a directory: {}", resolved.display()));
    }
    Ok(resolved)
}

/// Run in interactive mode - display UI, collect preferences, output to stdout.
fn run_interactive(path: &Path) -> Result<(), String> {
    println!();
    println!(
        "{}",
        style("╭────────────────────────────────────────────╮")
            .bold()
            .green()
    );
    println!(
        "{}",
        style("│          Pre-commit Starter                │")
            .bold()
            .green()
    );
    println!(
        "{}",
        style("│                                            │").green()
    );
    println!(
        "{}",
        style("│  Generate pre-commit config for your project  │").green()
    );
    println!(
        "{}",
        style("╰────────────────────────────────────────────╯")
            .bold()
            .green()
    );
    println!();

    println!("Analyzing project at: {}", style(path.display()).cyan());
    println!();

    // Detect technologies with spinner
    let spinner = ui::create_spinner("Detecting technologies...");
    let detected_config = discover::discover_config(path);
    spinner.finish_and_clear();

    // Display detected technologies
    ui::display_detected_technologies(&detected_config);

    // Ask if user wants to customize
    let customize = dialoguer::Confirm::new()
        .with_prompt("Would you like to customize the configuration?")
        .default(true)
        .interact()
        .unwrap_or(true);

    let final_config = if customize {
        ui::ask_user_preferences(detected_config)
    } else {
        detected_config
    };

    println!();

    // Generate configuration with spinner
    let spinner = ui::create_spinner("Generating pre-commit configuration...");
    let yaml = render::render_config(&final_config)?;
    spinner.finish_and_clear();

    // Output to stdout in interactive mode
    println!("{}", yaml);

    Ok(())
}

/// Run in auto-generate mode - detect, generate, save, and optionally run pre-commit.
fn run_auto(path: &Path, install: bool) -> Result<(), String> {
    run_auto_with_command(path, install, "pre-commit")
}

fn run_auto_with_command(path: &Path, install: bool, pre_commit_cmd: &str) -> Result<(), String> {
    // Detect technologies
    let config = discover::discover_config(path);

    // Generate YAML
    let yaml = render::render_config(&config)?;

    // Save configuration
    let config_file = path.join(".pre-commit-config.yaml");
    fs::write(&config_file, &yaml).map_err(|e| format!("Failed to write config: {}", e))?;
    ui::print_success(&format!(
        "Configuration saved to {}",
        style(config_file.display()).green()
    ));

    if !install {
        ui::print_info("Generated only. Pass --install to install hooks and run pre-commit.");
        return Ok(());
    }

    // Install pre-commit hooks
    match run_command(pre_commit_cmd, path, &["install"]) {
        Ok(output) => {
            if !output.status.success() {
                ui::print_error(&format!(
                    "Failed to install pre-commit hooks.\n{}",
                    format_command_output(&output)
                ));
                return Ok(());
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                ui::print_info("pre-commit not found in PATH. Run 'pre-commit install' manually.");
                return Ok(());
            }
            return Err(format!("Failed to run pre-commit: {}", e));
        }
    }

    // Run pre-commit on all files
    match run_command(pre_commit_cmd, path, &["run", "--all-files"]) {
        Ok(output) => {
            if output.status.success() {
                ui::print_success("Pre-commit setup complete and all hooks passed!");
            } else {
                ui::print_info("Pre-commit setup complete but some hooks failed:");
                println!("{}", format_command_output(&output));
            }
        }
        Err(e) => {
            return Err(format!("Failed to run pre-commit hooks: {}", e));
        }
    }

    Ok(())
}

fn run_command(command: &str, path: &Path, args: &[&str]) -> Result<Output, std::io::Error> {
    Command::new(command).args(args).current_dir(path).output()
}

fn format_command_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let mut parts = Vec::new();
    if let Some(code) = output.status.code() {
        parts.push(format!("Exit code: {}", code));
    } else {
        parts.push("Process terminated by signal".to_string());
    }
    if !stdout.is_empty() {
        parts.push(format!("stdout:\n{}", stdout));
    }
    if !stderr.is_empty() {
        parts.push(format!("stderr:\n{}", stderr));
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(unix)]
    fn create_fake_pre_commit(tmp: &Path, fail_install: bool) -> (String, std::path::PathBuf) {
        let script_path = tmp.join("fake-pre-commit.sh");
        let log_path = tmp.join("pre-commit-calls.log");
        let fail_flag = if fail_install { "1" } else { "0" };
        let script = format!(
            r#"#!/bin/sh
echo "$@" >> "{}"
if [ "$1" = "install" ] && [ "{}" = "1" ]; then
  echo "install failed" >&2
  exit 1
fi
if [ "$1" = "run" ]; then
  echo "run completed"
fi
exit 0
"#,
            log_path.display(),
            fail_flag
        );

        fs::write(&script_path, script).unwrap();
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        (script_path.to_string_lossy().into_owned(), log_path)
    }

    #[cfg(unix)]
    #[test]
    fn test_run_auto_default_skips_commands() {
        let tmp = tempdir().unwrap();
        let (cmd, calls_log) = create_fake_pre_commit(tmp.path(), false);

        let result = run_auto_with_command(tmp.path(), false, &cmd);
        assert!(result.is_ok());
        assert!(tmp.path().join(".pre-commit-config.yaml").exists());
        assert!(!calls_log.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_run_auto_stops_when_install_fails() {
        let tmp = tempdir().unwrap();
        let (cmd, calls_log) = create_fake_pre_commit(tmp.path(), true);

        let result = run_auto_with_command(tmp.path(), true, &cmd);
        assert!(result.is_ok());

        let calls = fs::read_to_string(calls_log).unwrap();
        assert_eq!(calls.lines().count(), 1);
        assert_eq!(calls.trim(), "install");
    }

    #[cfg(unix)]
    #[test]
    fn test_run_auto_runs_install_then_run() {
        let tmp = tempdir().unwrap();
        let (cmd, calls_log) = create_fake_pre_commit(tmp.path(), false);

        let result = run_auto_with_command(tmp.path(), true, &cmd);
        assert!(result.is_ok());

        let calls = fs::read_to_string(calls_log).unwrap();
        let lines: Vec<_> = calls.lines().collect();
        assert_eq!(lines, vec!["install", "run --all-files"]);
    }

    #[test]
    fn test_resolve_directory_path_rejects_file() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file.txt");
        fs::write(&file_path, "x").unwrap();

        let err = resolve_directory_path(&file_path).unwrap_err();
        assert!(err.contains("Path is not a directory"));
    }
}
