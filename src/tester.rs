//! Tester building and execution

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use which::which;

use crate::config::CoursePaths;

/// Requested build platform. Auto uses Go on the host when available, otherwise Docker.
#[derive(Clone, Copy, Debug)]
pub enum BuildTarget {
    Auto,
    Host,
    Linux,
}

/// Build the resolved platform, reusing a matching binary only when not forced.
/// The marker is updated only after a successful builder and verified output.
pub fn build_tester(course_paths: &CoursePaths, target: BuildTarget, force: bool) -> Result<()> {
    let use_docker = match target {
        BuildTarget::Auto => which("go").is_err(),
        BuildTarget::Host => {
            which("go").context("host tester build requires Go on PATH")?;
            false
        }
        BuildTarget::Linux => true,
    };
    course_paths.ensure_dirs()?;
    let os = if use_docker { "linux" } else { env::consts::OS };
    if !force
        && course_paths.tester_binary.exists()
        && fs::read_to_string(&course_paths.os_marker).is_ok_and(|marker| marker.trim() == os)
    {
        tracing::info!("Tester already built for {}", os);
        return Ok(());
    }
    let pkg = if course_paths.tester_src.join("cmd/tester").is_dir() {
        "./cmd/tester"
    } else {
        "."
    };
    if use_docker {
        build_with_docker(course_paths, pkg)?;
    } else {
        build_with_go(course_paths, pkg)?;
    }
    if !course_paths.tester_binary.exists() {
        anyhow::bail!(
            "tester binary not found at {}",
            course_paths.tester_binary.display()
        );
    }
    fs::write(&course_paths.os_marker, os)?;
    println!("{}", "tester built successfully".green());
    Ok(())
}

/// Resolve relative paths before changing a child's working directory. Missing
/// binaries remain previewable; real execution still reports the spawn failure.
fn absolute_path(path: &Path) -> Result<std::path::PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    Ok(absolute.canonicalize().unwrap_or(absolute))
}

fn build_with_go(course_paths: &CoursePaths, pkg: &str) -> Result<()> {
    println!("{}", "building tester with go...".dimmed());

    let pb = if crate::output::no_color() {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message(format!(
        "go build -o {} {}",
        course_paths.tester_binary.display(),
        pkg
    ));
    pb.enable_steady_tick(Duration::from_millis(100));

    let status = Command::new("go")
        .arg("build")
        .arg("-o")
        .arg(absolute_path(&course_paths.tester_binary)?)
        .arg(pkg)
        .current_dir(&course_paths.tester_src)
        .status()
        .context("running go build")?;

    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!(
            "go build failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn build_with_docker(course_paths: &CoursePaths, pkg: &str) -> Result<()> {
    println!(
        "{}",
        "building tester inside docker (linux binary)...".dimmed()
    );

    let src_mount = format!(
        "{}:/src",
        absolute_path(&course_paths.tester_src)?.display()
    );
    let out_mount = format!("{}:/out", absolute_path(&course_paths.bin)?.display());

    let pb = if crate::output::no_color() {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message("docker run golang:1.24-bookworm go build...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &src_mount,
            "-v",
            &out_mount,
            "-w",
            "/src",
            "-e",
            "CGO_ENABLED=0",
            "golang:1.24-bookworm",
            "go",
            "build",
            "-o",
            "/out/tester",
            pkg,
        ])
        .status()
        .context("running docker build")?;

    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!(
            "docker build failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// Test case for the tester
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestCase {
    pub slug: String,
    pub tester_log_prefix: String,
    pub title: String,
}

/// Build test cases JSON for the tester
pub fn build_test_cases(
    stages: &[&crate::course::Stage],
    upto_index: usize,
    only_current: bool,
) -> Vec<TestCase> {
    let indices: Vec<usize> = if only_current {
        vec![upto_index]
    } else {
        (0..=upto_index).collect()
    };

    indices
        .iter()
        .map(|&i| {
            let stage = stages[i];
            TestCase {
                slug: stage.slug.clone(),
                tester_log_prefix: format!("stage-{}", i + 1),
                title: format!("Stage #{}: {}", i + 1, stage.name),
            }
        })
        .collect()
}

/// Run the tester against the project
pub fn run_tester(
    project_root: &Path,
    tester_binary: &Path,
    test_cases: &[TestCase],
    docker_image: Option<&str>,
    dry_run: bool,
) -> Result<i32> {
    let test_cases_json = serde_json::to_string(test_cases)?;

    // Use absolute path for env vars (tester requires absolute paths)
    let project_root_abs = absolute_path(project_root)?;
    let tester_binary = absolute_path(tester_binary)?;

    let env_vars_host: Vec<(String, String)> = vec![
        (
            "CODECRAFTERS_REPOSITORY_DIR".to_string(),
            project_root_abs.to_string_lossy().to_string(),
        ),
        (
            "CODECRAFTERS_SUBMISSION_DIR".to_string(),
            project_root_abs.to_string_lossy().to_string(),
        ),
        (
            "CODECRAFTERS_TEST_CASES_JSON".to_string(),
            test_cases_json.clone(),
        ),
    ];

    // For Docker, env vars should point to container paths
    let env_vars_docker: Vec<(String, String)> = vec![
        (
            "CODECRAFTERS_REPOSITORY_DIR".to_string(),
            "/app".to_string(),
        ),
        (
            "CODECRAFTERS_SUBMISSION_DIR".to_string(),
            "/app".to_string(),
        ),
        (
            "CODECRAFTERS_TEST_CASES_JSON".to_string(),
            test_cases_json.clone(),
        ),
    ];

    let mut cmd = if let Some(image) = docker_image {
        // Verify tester is linux binary
        let os_marker = tester_binary.parent().unwrap().join("OS");
        if os_marker.exists() {
            let os = fs::read_to_string(&os_marker)?;
            if os.trim() != "linux" {
                anyhow::bail!(
                    "docker mode needs a linux tester binary (current: {}); rebuild with `craftcoders update --build-target linux`",
                    os.trim()
                );
            }
        }

        let mut cmd = Command::new("docker");
        cmd.args(["run", "--rm", "-i"])
            .arg("-v")
            .arg(format!("{}:/app", project_root_abs.display()))
            .arg("-v")
            .arg(format!(
                "{}:/usr/local/bin/tester:ro",
                tester_binary.display()
            ))
            .arg("-w")
            .arg("/app");

        for (k, v) in &env_vars_docker {
            cmd.arg("-e").arg(format!("{}={}", k, v));
        }

        cmd.arg(image).arg("tester");

        cmd
    } else {
        // Verify tester matches host OS
        let os_marker = tester_binary.parent().unwrap().join("OS");
        if os_marker.exists() {
            let os = fs::read_to_string(&os_marker)?;
            if os.trim() == "linux" && !cfg!(target_os = "linux") {
                anyhow::bail!(
                    "tester was built for linux (via docker); use --docker IMAGE, or install Go and run `craftcoders update --build-target host`"
                );
            }
        }

        let mut cmd = Command::new(&tester_binary);
        cmd.envs(env_vars_host).current_dir(&project_root_abs);
        cmd
    };

    if dry_run {
        // Command contains only intentional overrides, never enumerated ambient secrets.
        println!("command: {:?}", cmd);
        println!("test cases:\n{}", serde_json::to_string_pretty(test_cases)?);
        return Ok(0);
    }

    // Execute with inherited stdio for streaming output
    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("spawning tester")?;

    let status = child.wait().context("waiting for tester")?;
    Ok(status.code().unwrap_or(-1))
}

/// Check if tester binary exists and is usable
#[allow(dead_code)]
pub fn check_tester(course_paths: &CoursePaths) -> Result<()> {
    if !course_paths.tester_binary.exists() {
        anyhow::bail!("tester binary not found; run `craftcoders update` to build it");
    }
    Ok(())
}
