//! Discovery module for detecting project technologies.
//!
//! Scans a repository to detect what technologies are used based on
//! file extensions, filenames, and file contents.

use crate::config::PreCommitConfig;
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

// Technology detection indicators
const PYTHON_INDICATORS: &[&str] = &[
    "setup.py",
    "pyproject.toml",
    "requirements.txt",
    "pipfile",
    "poetry.lock",
    "setup.cfg",
    "tox.ini",
    "pytest.ini",
    ".py",
    "manage.py",
    "__init__.py",
];

const JAVASCRIPT_INDICATORS: &[&str] = &[
    "package.json",
    "yarn.lock",
    "package-lock.json",
    "npm-shrinkwrap.json",
    ".js",
    ".mjs",
    ".cjs",
    "webpack.config.js",
    "vite.config.js",
    "rollup.config.js",
    "babel.config.js",
    ".babelrc",
];

const TYPESCRIPT_INDICATORS: &[&str] = &[
    "tsconfig.json",
    "tsconfig.base.json",
    "tsconfig.build.json",
    ".ts",
    ".tsx",
    ".d.ts",
];

const JSX_INDICATORS: &[&str] = &[
    ".jsx",
    ".tsx",
    "next.config.js",
    "gatsby-config.js",
    "react-scripts",
    ".storybook",
];

const GO_INDICATORS: &[&str] = &["go.mod", "go.sum", "main.go", ".go"];

const DOCKER_INDICATORS: &[&str] = &[
    "dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    ".dockerignore",
    "dockerfile.dev",
    "dockerfile.prod",
];

const YAML_INDICATORS: &[&str] = &[".yml", ".yaml", "docker-compose.yml", "docker-compose.yaml"];
const JSON_INDICATORS: &[&str] = &[".json"];
const TOML_INDICATORS: &[&str] = &[".toml", "pyproject.toml"];
const XML_INDICATORS: &[&str] = &[".xml"];

const PRETTIER_CONFIG_FILES: &[&str] = &[
    ".prettierrc",
    ".prettierrc.json",
    ".prettierrc.yml",
    ".prettierrc.yaml",
    ".prettierrc.json5",
    ".prettierrc.js",
    ".prettierrc.cjs",
    ".prettierrc.mjs",
    "prettier.config.js",
    "prettier.config.cjs",
    "prettier.config.mjs",
];

const FLAT_ESLINT_CONFIG_FILES: &[&str] = &[
    "eslint.config.js",
    "eslint.config.mjs",
    "eslint.config.cjs",
    "eslint.config.ts",
    "eslint.config.mts",
    "eslint.config.cts",
];

#[derive(Debug, Default)]
struct ProjectFiles {
    names: HashSet<String>,
    extensions: HashSet<String>,
    relative_paths: HashSet<String>,
}

impl ProjectFiles {
    fn all_indicators(&self) -> HashSet<String> {
        self.names
            .iter()
            .chain(self.extensions.iter())
            .cloned()
            .collect()
    }

    fn has_path(&self, path: &str) -> bool {
        self.relative_paths.contains(path)
    }

    fn first_existing_path(&self, candidates: &[&str]) -> Option<String> {
        candidates.iter().find_map(|candidate| {
            self.relative_paths
                .get(&candidate.to_lowercase())
                .map(ToOwned::to_owned)
        })
    }

    fn has_file_in_dir_with_extension(&self, dir: &str, extensions: &[&str]) -> bool {
        let dir = dir.trim_end_matches('/');
        self.relative_paths.iter().any(|path| {
            path.strip_prefix(dir)
                .and_then(|rest| rest.strip_prefix('/'))
                .is_some_and(|rest| {
                    !rest.contains('/') && extensions.iter().any(|ext| rest.ends_with(ext))
                })
        })
    }
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect::<Vec<_>>()
        .join("/")
}

fn discover_project_files(path: &Path) -> ProjectFiles {
    let mut files = HashSet::new();
    let mut extensions = HashSet::new();
    let mut relative_paths = HashSet::new();

    let walker = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            files.insert(file_name);

            if let Some(ext) = entry.path().extension() {
                extensions.insert(format!(".{}", ext.to_string_lossy().to_lowercase()));
            }

            if let Ok(relative_path) = entry.path().strip_prefix(path) {
                relative_paths.insert(normalize_relative_path(relative_path));
            }
        }
    }

    ProjectFiles {
        names: files,
        extensions,
        relative_paths,
    }
}

/// Check if files contain any of the given indicators.
fn has_indicator(files: &HashSet<String>, indicators: &[&str]) -> bool {
    indicators
        .iter()
        .any(|ind| files.contains(&ind.to_lowercase()))
}

/// Detect if this is a Python project.
pub fn detect_python(files: &HashSet<String>) -> bool {
    has_indicator(files, PYTHON_INDICATORS)
}

/// Detect if project uses uv with uv.lock file.
pub fn detect_uv_lock(files: &HashSet<String>) -> bool {
    files.contains("uv.lock")
}

/// Detect if this is a JavaScript project.
pub fn detect_javascript(files: &HashSet<String>) -> bool {
    has_indicator(files, JAVASCRIPT_INDICATORS)
}

/// Detect if project uses TypeScript.
pub fn detect_typescript(files: &HashSet<String>) -> bool {
    has_indicator(files, TYPESCRIPT_INDICATORS)
}

/// Detect if project uses JSX/React.
pub fn detect_jsx(files: &HashSet<String>) -> bool {
    has_indicator(files, JSX_INDICATORS)
}

/// Detect if this is a Go project.
pub fn detect_go(files: &HashSet<String>) -> bool {
    has_indicator(files, GO_INDICATORS)
}

/// Detect if project uses Docker.
pub fn detect_docker(files: &HashSet<String>) -> bool {
    has_indicator(files, DOCKER_INDICATORS)
}

fn detect_github_actions_from_files(files: &ProjectFiles) -> bool {
    files.has_file_in_dir_with_extension(".github/workflows", &[".yml", ".yaml"])
}

/// Detect YAML files.
pub fn detect_yaml(files: &HashSet<String>) -> bool {
    has_indicator(files, YAML_INDICATORS)
}

/// Detect JSON files.
pub fn detect_json(files: &HashSet<String>) -> bool {
    has_indicator(files, JSON_INDICATORS)
}

/// Detect TOML files.
pub fn detect_toml(files: &HashSet<String>) -> bool {
    has_indicator(files, TOML_INDICATORS)
}

/// Detect XML files.
pub fn detect_xml(files: &HashSet<String>) -> bool {
    has_indicator(files, XML_INDICATORS)
}

/// Attempt to detect Python version from project files.
pub fn detect_python_version(path: &Path) -> Option<String> {
    // Prefer explicit interpreter pins over broad metadata ranges.
    let python_version_path = path.join(".python-version");
    if python_version_path.exists() {
        if let Ok(content) = fs::read_to_string(&python_version_path) {
            if let Some(version) = normalize_python_version(content.trim()) {
                return Some(version);
            }
        }
    }

    // Check pyproject.toml
    let pyproject_path = path.join("pyproject.toml");
    if pyproject_path.exists() {
        if let Ok(content) = fs::read_to_string(&pyproject_path) {
            if let Ok(parsed) = content.parse::<toml::Table>() {
                if let Some(project) = parsed.get("project").and_then(|p| p.as_table()) {
                    if let Some(requires_python) =
                        project.get("requires-python").and_then(|r| r.as_str())
                    {
                        // Only use exact interpreter-style constraints. Broad ranges like
                        // ">=3.11" describe compatibility, not the interpreter to run hooks with.
                        let re = Regex::new(r"^\s*(?:==\s*)?(\d+\.\d+(?:\.\d+)?)\s*$").unwrap();
                        if let Some(caps) = re.captures(requires_python) {
                            return Some(format!("python{}", &caps[1]));
                        }
                    }
                }
            }
        }
    }

    None
}

fn normalize_python_version(version: &str) -> Option<String> {
    if version.is_empty() {
        return None;
    }
    if version.starts_with("python") {
        Some(version.to_string())
    } else {
        Some(format!("python{}", version))
    }
}

/// Discover project configuration by analyzing files.
pub fn discover_config(path: &Path) -> PreCommitConfig {
    let project_files = discover_project_files(path);
    let files = project_files.all_indicators();

    let has_python = detect_python(&files);
    let has_js = detect_javascript(&files);
    let has_typescript = detect_typescript(&files);
    let has_jsx = detect_jsx(&files);
    let has_go = detect_go(&files);
    let has_docker = detect_docker(&files);
    let has_github_actions = detect_github_actions_from_files(&project_files);

    let has_yaml = detect_yaml(&files);
    let has_json = detect_json(&files);
    let has_toml = detect_toml(&files);
    let has_xml = detect_xml(&files);

    let python_version = if has_python {
        detect_python_version(path)
    } else {
        None
    };

    PreCommitConfig {
        python_version,
        yaml_check: has_yaml,
        json_check: has_json,
        toml_check: has_toml,
        xml_check: has_xml,
        case_conflict: true, // Always enable for cross-platform compatibility
        executables: true,   // Always enable for shell script safety
        symlinks: false,
        python_base: has_python,
        python: has_python,
        uv_lock: detect_uv_lock(&files),
        pyrefly: project_files.has_path("pyrefly.toml"),
        pyrefly_args: None,
        docker: has_docker,
        dockerfile_linting: true,
        dockerignore_check: false,
        github_actions: has_github_actions,
        workflow_validation: true,
        security_scanning: false,
        js: has_js,
        typescript: has_typescript,
        jsx: has_jsx,
        prettier_config: project_files.first_existing_path(PRETTIER_CONFIG_FILES),
        eslint_config: project_files.first_existing_path(FLAT_ESLINT_CONFIG_FILES),
        go: has_go,
        go_critic: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::tempdir;

    #[test]
    fn test_detect_python() {
        let mut files = HashSet::new();
        files.insert("pyproject.toml".to_string());
        assert!(detect_python(&files));
    }

    #[test]
    fn test_detect_python_by_extension() {
        let mut files = HashSet::new();
        files.insert(".py".to_string());
        assert!(detect_python(&files));
    }

    #[test]
    fn test_detect_javascript() {
        let mut files = HashSet::new();
        files.insert("package.json".to_string());
        assert!(detect_javascript(&files));
    }

    #[test]
    fn test_detect_go() {
        let mut files = HashSet::new();
        files.insert("go.mod".to_string());
        assert!(detect_go(&files));
    }

    #[test]
    fn test_detect_docker() {
        let mut files = HashSet::new();
        files.insert("dockerfile".to_string());
        assert!(detect_docker(&files));
    }

    #[test]
    fn test_no_false_positives() {
        let files = HashSet::new();
        assert!(!detect_python(&files));
        assert!(!detect_javascript(&files));
        assert!(!detect_go(&files));
        assert!(!detect_docker(&files));
    }

    #[test]
    fn test_detect_python_version_ignores_lower_bound_pyproject_range() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nrequires-python = \">=3.11,<4\"",
        )
        .unwrap();

        assert_eq!(detect_python_version(tmp.path()), None);
    }

    #[test]
    fn test_detect_python_version_from_exact_pyproject() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nrequires-python = \"==3.11\"",
        )
        .unwrap();

        assert_eq!(
            detect_python_version(tmp.path()),
            Some("python3.11".to_string())
        );
    }

    #[test]
    fn test_detect_python_version_exact_patch_version() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nrequires-python = \"3.10.5\"",
        )
        .unwrap();

        assert_eq!(
            detect_python_version(tmp.path()),
            Some("python3.10.5".to_string())
        );
    }

    #[test]
    fn test_detect_python_version_from_python_version_file() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join(".python-version"), "3.12.1\n").unwrap();

        assert_eq!(
            detect_python_version(tmp.path()),
            Some("python3.12.1".to_string())
        );
    }

    #[test]
    fn test_python_version_file_takes_precedence() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join(".python-version"), "3.12\n").unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nrequires-python = \"==3.11\"",
        )
        .unwrap();

        assert_eq!(
            detect_python_version(tmp.path()),
            Some("python3.12".to_string())
        );
    }

    #[test]
    fn test_discover_config_detects_pyrefly_only_when_config_exists() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("main.py"), "print('x')\n").unwrap();
        let config = discover_config(tmp.path());
        assert!(config.python);
        assert!(!config.pyrefly);

        fs::write(tmp.path().join("pyrefly.toml"), "").unwrap();
        let config = discover_config(tmp.path());
        assert!(config.pyrefly);
    }

    #[test]
    fn test_github_actions_detection_is_path_aware() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("docs")).unwrap();
        fs::write(tmp.path().join("docs").join("workflow.yml"), "name: docs\n").unwrap();
        assert!(!discover_config(tmp.path()).github_actions);

        fs::create_dir_all(tmp.path().join(".github").join("workflows")).unwrap();
        fs::write(
            tmp.path().join(".github").join("workflows").join("ci.yml"),
            "name: ci\n",
        )
        .unwrap();
        assert!(discover_config(tmp.path()).github_actions);
    }

    #[test]
    fn test_javascript_config_detection() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("package.json"), "{}\n").unwrap();

        let config = discover_config(tmp.path());
        assert!(config.js);
        assert_eq!(config.prettier_config, None);
        assert_eq!(config.eslint_config, None);

        fs::write(tmp.path().join(".prettierrc"), "{}\n").unwrap();
        fs::write(tmp.path().join("eslint.config.js"), "export default [];\n").unwrap();

        let config = discover_config(tmp.path());
        assert_eq!(config.prettier_config, Some(".prettierrc".to_string()));
        assert_eq!(config.eslint_config, Some("eslint.config.js".to_string()));
    }
}
