//! Terminal UI module for interactive mode.
//!
//! Provides Rich-like terminal output using console, dialoguer, and indicatif.

use crate::config::PreCommitConfig;
use console::{style, Term};
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Display detected technologies in a formatted table.
pub fn display_detected_technologies(config: &PreCommitConfig) {
    let term = Term::stdout();

    println!();
    println!(
        "{}",
        style("┌─────────────────────────────────────────────────┐").cyan()
    );
    println!(
        "{}",
        style("│           Detected Technologies                 │").cyan()
    );
    println!(
        "{}",
        style("├─────────────────────────────────────────────────┤").cyan()
    );

    // Language detections
    if config.python {
        let version_info = config
            .python_version
            .as_deref()
            .unwrap_or("No version specified");
        println!(
            "│  {} Python          {}  │",
            style("✓").green(),
            style(format!("({})", version_info)).dim()
        );
    }

    if config.js {
        let mut details = Vec::new();
        if config.typescript {
            details.push("TypeScript");
        }
        if config.jsx {
            details.push("JSX/React");
        }
        let detail_str = if details.is_empty() {
            "Basic JavaScript".to_string()
        } else {
            details.join(", ")
        };
        println!(
            "│  {} JavaScript      {}  │",
            style("✓").green(),
            style(format!("({})", detail_str)).dim()
        );
    }

    if config.go {
        println!(
            "│  {} Go                                           │",
            style("✓").green()
        );
    }

    // Infrastructure
    if config.docker {
        println!(
            "│  {} Docker                                       │",
            style("✓").green()
        );
    }

    if config.github_actions {
        println!(
            "│  {} GitHub Actions                               │",
            style("✓").green()
        );
    }

    // File types
    let mut file_types = Vec::new();
    if config.yaml_check {
        file_types.push("YAML");
    }
    if config.json_check {
        file_types.push("JSON");
    }
    if config.toml_check {
        file_types.push("TOML");
    }
    if config.xml_check {
        file_types.push("XML");
    }

    if !file_types.is_empty() {
        println!(
            "│  {} File types      {}  │",
            style("✓").green(),
            style(format!("({})", file_types.join(", "))).dim()
        );
    }

    println!(
        "{}",
        style("└─────────────────────────────────────────────────┘").cyan()
    );
    println!();
    let _ = term.flush();
}

/// Create a spinner for progress indication.
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Ask user for their preferences, using detected config as defaults.
pub fn ask_user_preferences(detected_config: PreCommitConfig) -> PreCommitConfig {
    let mut config = detected_config.clone();

    println!();
    println!("{}", style("Configure Pre-commit Hooks").bold().blue());
    println!("We'll ask you a few questions to customize your configuration.");
    println!(
        "{}",
        style("Detected defaults are shown - just press Enter to accept them.").dim()
    );
    println!();

    // Base hooks section
    println!("{}", style("Basic Hooks").bold());

    if detected_config.yaml_check {
        config.yaml_check = Confirm::new()
            .with_prompt("Include YAML syntax checking?")
            .default(detected_config.yaml_check)
            .interact()
            .unwrap_or(detected_config.yaml_check);
    }

    if detected_config.json_check {
        config.json_check = Confirm::new()
            .with_prompt("Include JSON syntax checking?")
            .default(detected_config.json_check)
            .interact()
            .unwrap_or(detected_config.json_check);
    }

    if detected_config.toml_check {
        config.toml_check = Confirm::new()
            .with_prompt("Include TOML syntax checking?")
            .default(detected_config.toml_check)
            .interact()
            .unwrap_or(detected_config.toml_check);
    }

    if detected_config.xml_check {
        config.xml_check = Confirm::new()
            .with_prompt("Include XML syntax checking?")
            .default(detected_config.xml_check)
            .interact()
            .unwrap_or(detected_config.xml_check);
    }

    // Always enable safety checks
    config.case_conflict = true;
    config.executables = true;

    println!();

    // Python section
    if detected_config.python {
        println!("{}", style("Python Hooks").bold());
        config.python = Confirm::new()
            .with_prompt("Include Python hooks (Ruff)?")
            .default(detected_config.python)
            .interact()
            .unwrap_or(detected_config.python);

        if config.python {
            config.python_base = detected_config.python_base;
            config.uv_lock = detected_config.uv_lock;
            config.pyrefly = Confirm::new()
                .with_prompt("Include Pyrefly type checking?")
                .default(detected_config.pyrefly)
                .interact()
                .unwrap_or(detected_config.pyrefly);
            if config.pyrefly {
                config.pyrefly_args = detected_config.pyrefly_args.clone();
            } else {
                config.pyrefly_args = None;
            }
        }
        println!();
    }

    // JavaScript section
    if detected_config.js {
        println!("{}", style("JavaScript/TypeScript Hooks").bold());
        config.js = Confirm::new()
            .with_prompt("Include JavaScript/TypeScript hooks (Prettier + ESLint)?")
            .default(detected_config.js)
            .interact()
            .unwrap_or(detected_config.js);

        if config.js {
            config.typescript = Confirm::new()
                .with_prompt("Include TypeScript support?")
                .default(detected_config.typescript)
                .interact()
                .unwrap_or(detected_config.typescript);

            config.jsx = Confirm::new()
                .with_prompt("Include JSX/React support?")
                .default(detected_config.jsx)
                .interact()
                .unwrap_or(detected_config.jsx);
        }
        println!();
    }

    // Go section
    if detected_config.go {
        println!("{}", style("Go Hooks").bold());
        config.go = Confirm::new()
            .with_prompt("Include Go hooks (golangci-lint + formatting)?")
            .default(detected_config.go)
            .interact()
            .unwrap_or(detected_config.go);

        if config.go {
            config.go_critic = Confirm::new()
                .with_prompt("Include go-critic for additional linting?")
                .default(false)
                .interact()
                .unwrap_or(false);
        }
        println!();
    }

    // Docker section
    if detected_config.docker {
        println!("{}", style("Docker Hooks").bold());
        config.docker = Confirm::new()
            .with_prompt("Include Docker hooks?")
            .default(detected_config.docker)
            .interact()
            .unwrap_or(detected_config.docker);

        if config.docker {
            config.dockerfile_linting = Confirm::new()
                .with_prompt("Include Dockerfile linting (hadolint)?")
                .default(true)
                .interact()
                .unwrap_or(true);
        }
        println!();
    }

    // GitHub Actions section
    if detected_config.github_actions {
        println!("{}", style("GitHub Actions Hooks").bold());
        config.github_actions = Confirm::new()
            .with_prompt("Include GitHub Actions hooks?")
            .default(detected_config.github_actions)
            .interact()
            .unwrap_or(detected_config.github_actions);

        if config.github_actions {
            config.workflow_validation = Confirm::new()
                .with_prompt("Include workflow validation (actionlint)?")
                .default(true)
                .interact()
                .unwrap_or(true);

            config.security_scanning = Confirm::new()
                .with_prompt("Include security scanning for workflows?")
                .default(false)
                .interact()
                .unwrap_or(false);
        }
        println!();
    }

    // Python version
    if config.python {
        if let Some(ref detected_version) = detected_config.python_version {
            let version: String = Input::new()
                .with_prompt("Python version for default_language_version")
                .default(detected_version.clone())
                .interact_text()
                .unwrap_or_else(|_| detected_version.clone());
            config.python_version = Some(version);
        } else {
            let version: String = Input::new()
                .with_prompt(
                    "Python version for default_language_version (optional, press Enter to skip)",
                )
                .allow_empty(true)
                .interact_text()
                .unwrap_or_default();
            config.python_version = if version.is_empty() {
                None
            } else {
                Some(version)
            };
        }
    }

    config
}

/// Print a success message.
pub fn print_success(message: &str) {
    println!("{} {}", style("✓").green().bold(), message);
}

/// Print an error message.
pub fn print_error(message: &str) {
    println!("{} {}", style("✗").red().bold(), message);
}

/// Print an info message.
pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ").blue().bold(), message);
}
