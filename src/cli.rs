//! CLI module for argument parsing.
//!
//! Uses clap with derive macros for simple, declarative CLI definition.

use clap::Parser;
use std::path::PathBuf;

/// Auto-detect technologies and generate pre-commit configuration files.
#[derive(Parser, Debug)]
#[command(name = "prec-templ")]
#[command(author = "Jakub Kriz")]
#[command(version)]
#[command(about = "Auto-detect technologies and generate pre-commit configuration files.", long_about = None)]
pub struct Cli {
    /// Enable interactive mode for customizing configuration
    #[arg(short, long)]
    pub interactive: bool,

    /// Generate .pre-commit-config.yaml without installing or running hooks
    #[arg(long, conflicts_with = "interactive")]
    pub generate_only: bool,

    /// After generating, run pre-commit install and pre-commit run --all-files
    #[arg(long, conflicts_with = "interactive", conflicts_with = "generate_only")]
    pub install: bool,

    /// Path to analyze (default: current directory)
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
}

impl Cli {
    /// Parse command-line arguments.
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let cli = Cli::parse_from(["prec-templ"]);
        assert!(!cli.interactive);
        assert!(!cli.generate_only);
        assert!(!cli.install);
        assert_eq!(cli.path, PathBuf::from("."));
    }

    #[test]
    fn test_interactive_short() {
        let cli = Cli::parse_from(["prec-templ", "-i"]);
        assert!(cli.interactive);
    }

    #[test]
    fn test_interactive_long() {
        let cli = Cli::parse_from(["prec-templ", "--interactive"]);
        assert!(cli.interactive);
    }

    #[test]
    fn test_custom_path() {
        let cli = Cli::parse_from(["prec-templ", "--path", "/some/path"]);
        assert_eq!(cli.path, PathBuf::from("/some/path"));
    }

    #[test]
    fn test_generate_only() {
        let cli = Cli::parse_from(["prec-templ", "--generate-only"]);
        assert!(cli.generate_only);
        assert!(!cli.interactive);
        assert!(!cli.install);
    }

    #[test]
    fn test_install() {
        let cli = Cli::parse_from(["prec-templ", "--install"]);
        assert!(cli.install);
        assert!(!cli.interactive);
        assert!(!cli.generate_only);
    }
}
