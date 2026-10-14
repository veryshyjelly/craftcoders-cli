//! Global configuration and paths

use anyhow::{Context, Result};
use dirs::home_dir;
use std::path::{Path, PathBuf};

/// Default base directory for courses and testers
const DEFAULT_CC_HOME: &str = ".craftcoders";

/// Resolve the CC_HOME directory
pub fn cc_home(override_dir: Option<&Path>) -> Result<PathBuf> {
    if let Some(dir) = override_dir {
        return Ok(dir.to_path_buf());
    }
    if let Ok(env) = std::env::var("CC_HOME") {
        return Ok(PathBuf::from(env));
    }
    let home = home_dir().context("could not determine home directory")?;
    Ok(home.join(DEFAULT_CC_HOME))
}

/// Paths within CC_HOME for a given course
pub struct CoursePaths {
    #[allow(dead_code)]
    pub base: PathBuf,
    pub definition: PathBuf,
    pub tester_src: PathBuf,
    pub bin: PathBuf,
    pub tester_binary: PathBuf,
    pub os_marker: PathBuf,
}

impl CoursePaths {
    pub fn new(cc_home: &Path, course: &str) -> Self {
        let base = cc_home.join("courses").join(course);
        Self {
            definition: base.join("definition"),
            tester_src: base.join("tester"),
            bin: base.join("bin"),
            tester_binary: base.join("bin").join("tester"),
            os_marker: base.join("bin").join("OS"),
            base,
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.definition)
            .with_context(|| format!("creating {}", self.definition.display()))?;
        std::fs::create_dir_all(&self.tester_src)
            .with_context(|| format!("creating {}", self.tester_src.display()))?;
        std::fs::create_dir_all(&self.bin)
            .with_context(|| format!("creating {}", self.bin.display()))?;
        Ok(())
    }
}

/// Project configuration file name
pub const PROJECT_FILE: &str = ".craftcoders.json";

/// Find the project root by looking for .craftcoders.json
pub fn find_project_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let project_file = current.join(PROJECT_FILE);
        if project_file.exists() {
            return Ok(current);
        }
        if !current.pop() {
            anyhow::bail!("not inside a craftcoders project (no {} found). Run `craftcoders init <course>` first.", PROJECT_FILE);
        }
    }
}