//! Project state management (.craftcoders.json)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::config::PROJECT_FILE;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub course: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub completed: Vec<String>,
}

impl ProjectConfig {
    pub fn new(course: String) -> Self {
        Self {
            course,
            extensions: Vec::new(),
            completed: Vec::new(),
        }
    }
}

pub fn load_project(project_root: &Path) -> Result<ProjectConfig> {
    let path = project_root.join(PROJECT_FILE);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let config: ProjectConfig = serde_json::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(config)
}

pub fn save_project(project_root: &Path, config: &ProjectConfig) -> Result<()> {
    let path = project_root.join(PROJECT_FILE);
    let content = serde_json::to_string_pretty(config)
        .context("serializing project config")?;
    fs::write(&path, content)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn init_project(project_root: &Path, course: String) -> Result<()> {
    fs::create_dir_all(project_root)
        .with_context(|| format!("creating project directory {}", project_root.display()))?;

    let config = ProjectConfig::new(course);
    save_project(project_root, &config)?;

    // Create codecrafters.yml
    let codecrafters_yml = project_root.join("codecrafters.yml");
    fs::write(&codecrafters_yml, r#"# Read by the tester. Set debug to true for verbose logs.
debug: false
"#).with_context(|| format!("writing {}", codecrafters_yml.display()))?;

    // Create your_program.sh entry point
    let entry = project_root.join("your_program.sh");
    let entry_content = r#"#!/bin/sh
# The tester runs this file to start your program.
# It will first run compile.sh if it exists, then launch your implementation.

# Run compile script if it exists
if [ -f "./compile.sh" ]; then
    echo "Running compile.sh..."
    ./compile.sh || {
        echo "Compilation failed!" >&2
        exit 1
    }
fi

# Replace the line below with whatever runs your implementation, e.g.
#   exec python3 -u app/main.py "$@"
#   exec go run ./app "$@"
#   exec java -jar target.jar "$@"
echo "edit your_program.sh to start your program" >&2
exit 1
"#;
    fs::write(&entry, entry_content)
        .with_context(|| format!("writing {}", entry.display()))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&entry)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&entry, perms)?;
    }

    // Initialize git repo if available
    if which::which("git").is_ok() {
        let _ = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(project_root)
            .status();
    }

    Ok(())
}