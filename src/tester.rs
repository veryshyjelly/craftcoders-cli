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

/// Build the official tester binary
pub fn build_tester(course_paths: &CoursePaths, _course: &str, use_docker: bool) -> Result<()> {
    course_paths.ensure_dirs()?;

    let pkg = if course_paths.tester_src.join("cmd").join("tester").is_dir() {
        "./cmd/tester"
    } else {
        "."
    };

    // Check if already built for this platform
    if course_paths.tester_binary.exists() {
        if let Ok(os) = fs::read_to_string(&course_paths.os_marker) {
            let current_os = if use_docker { "linux" } else { std::env::consts::OS };
            if os.trim() == current_os {
                tracing::info!("Tester already built for {}", current_os);
                return Ok(());
            }
        }
    }

    if use_docker || which("go").is_err() {
        build_with_docker(course_paths, pkg)?;
    } else {
        build_with_go(course_paths, pkg)?;
    }

    // Verify binary exists
    if !course_paths.tester_binary.exists() {
        anyhow::bail!("tester binary not found at {}", course_paths.tester_binary.display());
    }

    // Write OS marker
    let os = if use_docker { "linux" } else { std::env::consts::OS };
    fs::write(&course_paths.os_marker, os)?;

    Ok(())
}

fn build_with_go(course_paths: &CoursePaths, pkg: &str) -> Result<()> {
    println!("{}", "building tester with go...".dimmed());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
    pb.set_message(format!("go build -o {} {}", course_paths.tester_binary.display(), pkg));
    pb.enable_steady_tick(Duration::from_millis(100));

    let status = Command::new("go")
        .args(["build", "-o", course_paths.tester_binary.to_str().unwrap(), pkg])
        .current_dir(&course_paths.tester_src)
        .status()
        .context("running go build")?;

    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("go build failed with exit code: {}", status.code().unwrap_or(-1));
    }

    println!("{}", "tester built successfully".green());
    Ok(())
}

fn build_with_docker(course_paths: &CoursePaths, pkg: &str) -> Result<()> {
    println!("{}", "go not found; building tester inside docker (linux binary)...".dimmed());

    let src_mount = format!("{}:/src", course_paths.tester_src.display());
    let out_mount = format!("{}:/out", course_paths.bin.display());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
    pb.set_message("docker run golang:1.24-bookworm go build...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let status = Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &src_mount,
            "-v", &out_mount,
            "-w", "/src",
            "-e", "CGO_ENABLED=0",
            "golang:1.24-bookworm",
            "go", "build", "-o", "/out/tester", pkg,
        ])
        .status()
        .context("running docker build")?;

    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("docker build failed with exit code: {}", status.code().unwrap_or(-1));
    }

    println!("{}", "tester built successfully (linux)".green());
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

    indices.iter().map(|&i| {
        let stage = stages[i];
        TestCase {
            slug: stage.slug.clone(),
            tester_log_prefix: format!("stage-{}", i + 1),
            title: format!("Stage #{}: {}", i + 1, stage.name),
        }
    }).collect()
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
    let project_root_abs = project_root.canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());

    let env_vars_host: Vec<(String, String)> = vec![
        ("CODECRAFTERS_REPOSITORY_DIR".to_string(), project_root_abs.to_string_lossy().to_string()),
        ("CODECRAFTERS_SUBMISSION_DIR".to_string(), project_root_abs.to_string_lossy().to_string()),
        ("CODECRAFTERS_TEST_CASES_JSON".to_string(), test_cases_json.clone()),
    ];

    // For Docker, env vars should point to container paths
    let env_vars_docker: Vec<(String, String)> = vec![
        ("CODECRAFTERS_REPOSITORY_DIR".to_string(), "/app".to_string()),
        ("CODECRAFTERS_SUBMISSION_DIR".to_string(), "/app".to_string()),
        ("CODECRAFTERS_TEST_CASES_JSON".to_string(), test_cases_json.clone()),
    ];

    let (mut cmd, _env) = if let Some(image) = docker_image {
        // Verify tester is linux binary
        let os_marker = tester_binary.parent().unwrap().join("OS");
        if os_marker.exists() {
            let os = fs::read_to_string(&os_marker)?;
            if os.trim() != "linux" {
                anyhow::bail!(
                    "docker mode needs a linux tester binary (current: {}); rebuild with `cc update` or build on linux",
                    os.trim()
                );
            }
        }

        let mut cmd = Command::new("docker");
        cmd.args(["run", "--rm", "-i"])
            .arg("-v").arg(format!("{}:/app", project_root_abs.display()))
            .arg("-v").arg(format!("{}:/usr/local/bin/tester:ro", tester_binary.display()))
            .arg("-w").arg("/app");

        for (k, v) in &env_vars_docker {
            cmd.arg("-e").arg(format!("{}={}", k, v));
        }

        cmd.arg(image).arg("tester");

        (cmd, None::<Vec<(String, String)>>)
    } else {
        // Verify tester matches host OS
        let os_marker = tester_binary.parent().unwrap().join("OS");
        if os_marker.exists() {
            let os = fs::read_to_string(&os_marker)?;
            if os.trim() == "linux" && !cfg!(target_os = "linux") {
                anyhow::bail!(
                    "tester was built for linux (via docker); use --docker IMAGE, or install Go and run `craftcoders update`"
                );
            }
        }

        let mut cmd = Command::new(tester_binary);
        let mut env = env::vars().collect::<Vec<_>>();
        for (k, v) in &env_vars_host {
            env.push((k.clone(), v.clone()));
        }
        cmd.envs(env);
        (cmd, None::<Vec<(String, String)>>)
    };

    if dry_run {
        let cmd_str = format!("{:?}", cmd);
        let display = if cmd_str.len() > 120 {
            format!("{} ...", &cmd_str[..120])
        } else {
            cmd_str
        };
        println!("command: {}", display);
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
        anyhow::bail!("tester binary not found; run `cc update` to build it");
    }
    Ok(())
}