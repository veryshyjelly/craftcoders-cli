//! Git operations for course and tester repositories

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::Path;
use std::process::Command;

const ORG_URL: &str = "https://github.com/codecrafters-io";

/// Clone a repository with depth 1
pub fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(()); // Already cloned
    }

    println!("{} {}", "cloning".dimmed(), url);

    let status = Command::new("git")
        .args(["clone", "--depth", "1", url, dest.to_str().unwrap()])
        .status()
        .with_context(|| format!("cloning {}", url))?;

    if !status.success() {
        anyhow::bail!("git clone failed for {}", url);
    }

    Ok(())
}

/// Pull latest changes (fast-forward only)
pub fn pull_repo(dir: &Path) -> Result<()> {
    println!("{} {}", "updating".dimmed(), dir.display());

    let status = Command::new("git")
        .args(["pull", "--ff-only", "-q"])
        .current_dir(dir)
        .status()
        .with_context(|| format!("pulling in {}", dir.display()))?;

    if !status.success() {
        anyhow::bail!("git pull failed in {}", dir.display());
    }

    Ok(())
}

/// Ensure course and tester repositories exist
pub fn ensure_repos(
    cc_home: &Path,
    course: &str,
    tester_repo_override: Option<&str>,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let course_dir = cc_home.join("courses").join(course);
    let definition_dir = course_dir.join("definition");
    let tester_dir = course_dir.join("tester");

    let tester_repo = tester_repo_override.unwrap_or(&format!("{}-tester", course)).to_string();

    clone_repo(&format!("{}/build-your-own-{}.git", ORG_URL, course), &definition_dir)?;
    clone_repo(&format!("{}/{}.git", ORG_URL, tester_repo), &tester_dir)?;

    Ok((definition_dir, tester_dir))
}

/// Update course and tester repositories
pub fn update_repos(cc_home: &Path, course: &str) -> Result<()> {
    let course_dir = cc_home.join("courses").join(course);
    let definition_dir = course_dir.join("definition");
    let tester_dir = course_dir.join("tester");

    pull_repo(&definition_dir)?;
    pull_repo(&tester_dir)?;

    Ok(())
}

/// Get repository URL for a course
pub fn course_repo_url(course: &str) -> String {
    format!("{}/build-your-own-{}.git", ORG_URL, course)
}

/// Get tester repository URL
pub fn tester_repo_url(course: &str, override_repo: Option<&str>) -> String {
    format!("{}/{}.git", ORG_URL, override_repo.unwrap_or(&format!("{}-tester", course)))
}