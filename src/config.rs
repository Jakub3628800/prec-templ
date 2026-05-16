//! Configuration module for prec-templ generation.
//!
//! This module defines the configuration structure that controls which
//! pre-commit hooks are generated, equivalent to the Python Pydantic model.

use serde::{Deserialize, Serialize};

/// Configuration for pre-commit hook generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCommitConfig {
    /// Python version for default_language_version (e.g., "python3.11")
    #[serde(default)]
    pub python_version: Option<String>,

    // Base hooks options
    #[serde(default)]
    pub yaml_check: bool,
    #[serde(default)]
    pub json_check: bool,
    #[serde(default)]
    pub toml_check: bool,
    #[serde(default)]
    pub xml_check: bool,
    #[serde(default)]
    pub case_conflict: bool,
    #[serde(default)]
    pub executables: bool,
    #[serde(default)]
    pub symlinks: bool,
    #[serde(default)]
    pub python_base: bool,

    // Python hooks options
    #[serde(default)]
    pub python: bool,
    #[serde(default)]
    pub uv_lock: bool,
    #[serde(default)]
    pub pyrefly: bool,
    #[serde(default)]
    pub pyrefly_args: Option<Vec<String>>,

    // Docker hooks options
    #[serde(default)]
    pub docker: bool,
    #[serde(default = "default_true")]
    pub dockerfile_linting: bool,
    #[serde(default)]
    pub dockerignore_check: bool,

    // GitHub Actions hooks options
    #[serde(default)]
    pub github_actions: bool,
    #[serde(default = "default_true")]
    pub workflow_validation: bool,
    #[serde(default)]
    pub security_scanning: bool,

    // JavaScript hooks options
    #[serde(default)]
    pub js: bool,
    #[serde(default)]
    pub typescript: bool,
    #[serde(default)]
    pub jsx: bool,
    #[serde(default)]
    pub prettier_config: Option<String>,
    #[serde(default)]
    pub eslint_config: Option<String>,

    // Go hooks options
    #[serde(default)]
    pub go: bool,
    #[serde(default)]
    pub go_critic: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PreCommitConfig {
    fn default() -> Self {
        Self {
            python_version: None,
            yaml_check: false,
            json_check: false,
            toml_check: false,
            xml_check: false,
            case_conflict: false,
            executables: false,
            symlinks: false,
            python_base: false,
            python: false,
            uv_lock: false,
            pyrefly: false,
            pyrefly_args: None,
            docker: false,
            dockerfile_linting: true, // Default to true
            dockerignore_check: false,
            github_actions: false,
            workflow_validation: true, // Default to true
            security_scanning: false,
            js: false,
            typescript: false,
            jsx: false,
            prettier_config: None,
            eslint_config: None,
            go: false,
            go_critic: false,
        }
    }
}

impl PreCommitConfig {
    /// Create a new empty configuration.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref version) = self.python_version {
            if !version.starts_with("python") {
                return Err(format!(
                    "Python version must start with 'python' (e.g., python3.11), got: {}",
                    version
                ));
            }
        }
        Ok(())
    }

    /// Get list of detected technologies for display.
    pub fn detected_technologies(&self) -> Vec<&'static str> {
        let mut techs = Vec::new();
        if self.python {
            techs.push("python");
        }
        if self.js {
            techs.push("javascript");
        }
        if self.typescript {
            techs.push("typescript");
        }
        if self.go {
            techs.push("go");
        }
        if self.docker {
            techs.push("docker");
        }
        if self.github_actions {
            techs.push("github-actions");
        }
        techs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PreCommitConfig::default();
        assert!(!config.python);
        assert!(!config.js);
        assert!(config.dockerfile_linting); // Should default to true
        assert!(config.workflow_validation); // Should default to true
    }

    #[test]
    fn test_validate_python_version_valid() {
        let config = PreCommitConfig {
            python_version: Some("python3.11".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_python_version_invalid() {
        let config = PreCommitConfig {
            python_version: Some("3.11".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_detected_technologies() {
        let config = PreCommitConfig {
            python: true,
            js: true,
            docker: true,
            ..Default::default()
        };
        let techs = config.detected_technologies();
        assert!(techs.contains(&"python"));
        assert!(techs.contains(&"javascript"));
        assert!(techs.contains(&"docker"));
    }
}
