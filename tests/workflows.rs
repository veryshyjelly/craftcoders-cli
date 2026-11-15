//! Unix subprocess fixtures: no shared cwd/environment or real external tools.
#![cfg(unix)]

use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const DEFINITION: &str = r#"
name: Fixture Course
slug: demo
stages:
  - { name: First, slug: one, marketing_md: First instructions }
  - { name: Second, slug: two, marketing_md: Second instructions }
  - { name: Third, slug: three, marketing_md: Third instructions }
  - { name: Extra A, slug: extra-a, primary_extension_slug: alpha }
  - { name: Extra B, slug: extra-b, primary_extension_slug: beta }
extensions:
  - { slug: alpha, name: Alpha }
  - { slug: beta, name: Beta }
"#;

fn executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Fixture {
    temp: TempDir,
    work: PathBuf,
    cache: PathBuf,
    tools: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let work = base.join("workspace");
        let cache = base.join("cache");
        let tools = base.join("tools");
        for path in [
            &work,
            &tools,
            &temp.path().join("home"),
            &work.join("nested/deep"),
        ] {
            fs::create_dir_all(path).unwrap();
        }
        let fixture = Self {
            temp,
            work,
            cache,
            tools,
        };
        fixture.install_cache();
        fixture.save(json!({"course":"demo","extensions":[],"completed":[]}));
        executable(
            &fixture.work.join("compile.sh"),
            "pwd > compiled\nexit \"${COMPILE_EXIT:-0}\"",
        );
        fixture
    }

    fn install_cache(&self) {
        let course = self.cache.join("courses/demo");
        for name in ["definition", "tester", "bin"] {
            fs::create_dir_all(course.join(name)).unwrap();
        }
        fs::write(course.join("definition/course-definition.yml"), DEFINITION).unwrap();
        executable(
            &course.join("bin/tester"),
            r#"pwd > "$CAPTURE/cwd"
printf '%s' "$CODECRAFTERS_REPOSITORY_DIR" > "$CAPTURE/repository"
printf '%s' "$CODECRAFTERS_SUBMISSION_DIR" > "$CAPTURE/submission"
printf '%s' "$CODECRAFTERS_TEST_CASES_JSON" > "$CAPTURE/cases"
exit "${TESTER_EXIT:-0}""#,
        );
        fs::write(course.join("bin/OS"), std::env::consts::OS).unwrap();
    }

    fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_craftcoders"));
        command
            .env_clear()
            .current_dir(cwd)
            .env("PATH", &self.tools)
            .env("HOME", self.temp.path().join("home"))
            .env("CC_HOME", &self.cache)
            .env("CAPTURE", self.temp.path())
            .arg("--no-color");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(&self.work).args(args).output().unwrap()
    }

    fn config_bytes(&self) -> Vec<u8> {
        fs::read(self.work.join(".craftcoders.json")).unwrap()
    }

    fn config(&self) -> Value {
        serde_json::from_slice(&self.config_bytes()).unwrap()
    }

    fn save(&self, value: Value) {
        fs::write(
            self.work.join(".craftcoders.json"),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn tool(&self, name: &str, body: &str) {
        executable(&self.tools.join(name), body);
    }

    fn assert_no_execution(&self) {
        assert!(!self.work.join("compiled").exists());
        assert!(!self.temp.path().join("cases").exists());
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        stdout(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn nested_commands_use_root_and_invocation_relative_cache_with_cli_precedence() {
    let f = Fixture::new();
    let nested = f.work.join("nested/deep");
    for args in [
        vec!["status"],
        vec!["show", "--raw"],
        vec!["test"],
        vec!["submit"],
    ] {
        success(
            f.command(&nested)
                .env("CC_HOME", "missing-environment-cache")
                .args(["--cc-home", "../../../cache"])
                .args(args)
                .output()
                .unwrap(),
        );
    }
    assert_eq!(
        fs::read_to_string(f.temp.path().join("cwd"))
            .unwrap()
            .trim(),
        f.work.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(f.work.join("compiled")).unwrap().trim(),
        f.work.to_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(f.temp.path().join("repository")).unwrap(),
        f.work.to_str().unwrap()
    );
    assert_eq!(f.config()["completed"], json!(["one"]));
    success(
        f.command(&nested)
            .env("CC_HOME", "../../../cache")
            .arg("status")
            .output()
            .unwrap(),
    );
}

#[test]
fn legacy_minimal_config_loads_without_rewriting() {
    let f = Fixture::new();
    f.save(json!({"course":"demo"}));
    let before = f.config_bytes();
    success(f.run(&["status"]));
    success(f.run(&["show", "--json"]));
    success(f.run(&["test", "--dry-run"]));
    assert_eq!(f.config_bytes(), before);
}

#[test]
fn previews_and_invalid_selectors_have_no_side_effects() {
    let f = Fixture::new();
    let before = f.config_bytes();
    for command in ["test", "submit"] {
        success(
            f.command(&f.work)
                .env("COMPILE_EXIT", "42")
                .args([command, "--dry-run", "--upto", "two"])
                .output()
                .unwrap(),
        );
        for selector in ["0", "99", "missing", "extra-a"] {
            let output = f.run(&[command, "--upto", selector]);
            assert!(!output.status.success());
            assert!(String::from_utf8_lossy(&output.stderr).contains("stage not found"));
        }
        f.assert_no_execution();
        assert_eq!(f.config_bytes(), before);
    }
}

#[test]
fn compile_failure_stops_test_and_submit_before_tester_and_progress() {
    let f = Fixture::new();
    let before = f.config_bytes();
    for command in ["test", "submit"] {
        let output = f
            .command(&f.work)
            .env("COMPILE_EXIT", "42")
            .arg(command)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("compile.sh failed with exit code: 42")
        );
        assert!(!f.temp.path().join("cases").exists());
        assert_eq!(f.config_bytes(), before);
    }
}

#[test]
fn tester_exit_codes_and_target_only_progress_are_preserved() {
    let f = Fixture::new();
    let before = f.config_bytes();
    for command in ["test", "submit"] {
        let output = f
            .command(&f.work)
            .env("TESTER_EXIT", "23")
            .arg(command)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(f.config_bytes(), before);
    }
    success(f.run(&["test", "--upto", "3"]));
    assert_eq!(f.config_bytes(), before);
    let output = success(f.run(&["submit", "--upto", "3"]));
    assert_eq!(f.config()["completed"], json!(["three"]));
    assert!(stdout(&output).contains("Next: stage 1 - First"));
    let saved = f.config_bytes();
    success(f.run(&["submit", "--upto", "three"]));
    assert_eq!(f.config_bytes(), saved);
    success(f.run(&["submit"]));
    assert_eq!(f.config()["completed"], json!(["three", "one"]));
    let output = success(f.run(&["submit"]));
    assert_eq!(f.config()["completed"], json!(["three", "one", "two"]));
    assert!(stdout(&output).contains("last enabled stage"));
}

#[test]
fn explicit_retest_after_completion_and_prefix_payloads() {
    let f = Fixture::new();
    f.save(json!({"course":"demo","completed":["one","two","three"]}));
    let before = f.config_bytes();
    for args in [vec!["test"], vec!["submit"], vec!["submit", "--upto", "1"]] {
        let output = f.run(&args);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("all stages are complete"));
        f.assert_no_execution();
    }
    success(f.run(&["test", "--upto", "2"]));
    let cases: Value =
        serde_json::from_slice(&fs::read(f.temp.path().join("cases")).unwrap()).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 2);
    assert_eq!(cases[0]["slug"], "one");
    assert_eq!(cases[1]["slug"], "two");
    success(f.run(&["test", "--upto", "3", "--only-current"]));
    let cases: Value =
        serde_json::from_slice(&fs::read(f.temp.path().join("cases")).unwrap()).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 1);
    assert_eq!(cases[0]["slug"], "three");
    assert_eq!(f.config_bytes(), before);
}

#[test]
fn init_rejects_occupied_destinations_before_any_tool_or_cache_work() {
    let f = Fixture::new();
    for name in ["git", "go", "docker"] {
        f.tool(
            name,
            "printf '%s\\n' invoked >> \"$CAPTURE/tool-calls\"\nexit 1",
        );
    }
    let directory = f.temp.path().join("occupied");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("keep"), "original").unwrap();
    let file = f.temp.path().join("file");
    fs::write(&file, "original file").unwrap();
    for target in [&directory, &file] {
        let output = f
            .command(f.temp.path())
            .env("CC_HOME", "nonexistent-cache")
            .args(["init", "demo"])
            .arg(target)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!f.temp.path().join("tool-calls").exists());
        assert!(!f.temp.path().join("nonexistent-cache").exists());
    }
    assert_eq!(
        fs::read_to_string(directory.join("keep")).unwrap(),
        "original"
    );
    assert_eq!(fs::read_to_string(file).unwrap(), "original file");
}

#[test]
fn init_accepts_empty_and_absent_targets_without_building() {
    let f = Fixture::new();
    for name in ["go", "docker"] {
        f.tool(name, "echo unexpected >> \"$CAPTURE/build-calls\"\nexit 1");
    }
    let empty = f.temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    for target in [&empty, &f.temp.path().join("absent")] {
        success(
            f.command(f.temp.path())
                .args(["init", "demo", "--no-build"])
                .arg(target)
                .output()
                .unwrap(),
        );
        let config: Value =
            serde_json::from_slice(&fs::read(target.join(".craftcoders.json")).unwrap()).unwrap();
        assert_eq!(
            config,
            json!({"course":"demo","extensions":[],"completed":[]})
        );
        for name in ["compile.sh", "your_program.sh", "codecrafters.yml"] {
            assert!(target.join(name).exists());
        }
        for name in ["compile.sh", "your_program.sh"] {
            assert_ne!(
                fs::metadata(target.join(name))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111,
                0
            );
        }
    }
    assert!(!f.temp.path().join("build-calls").exists());
}

#[test]
fn relative_initially_missing_cache_is_absolute_during_clone_and_build() {
    let f = Fixture::new();
    fs::write(f.temp.path().join("definition-template"), DEFINITION).unwrap();
    f.tool("git", r#"printf '%s\n' "$*" >> "$CAPTURE/git-calls"
if [ "$1" = clone ]; then
  for destination in "$@"; do :; done
  case "$destination" in /*) ;; *) exit 90 ;; esac
  /bin/mkdir -p "$destination"
  case "$destination" in */definition) /bin/cp "$CAPTURE/definition-template" "$destination/course-definition.yml" ;; esac
fi"#);
    f.tool(
        "go",
        r#"printf '%s\n' "$*" >> "$CAPTURE/go-calls"
[ "$1" = build ] || exit 91
case "$3" in /*) ;; *) exit 92 ;; esac
printf '#!/bin/sh\nexit 0\n' > "$3"
/bin/chmod +x "$3""#,
    );
    let output = f
        .command(f.temp.path())
        .env("CC_HOME", "fresh-cache")
        .args(["init", "demo", "fresh-work"])
        .output()
        .unwrap();
    success(output);
    assert!(
        f.temp
            .path()
            .join("fresh-cache/courses/demo/bin/tester")
            .exists()
    );
    assert!(
        fs::read_to_string(f.temp.path().join("git-calls"))
            .unwrap()
            .contains(f.temp.path().join("fresh-cache").to_str().unwrap())
    );
    assert!(
        fs::read_to_string(f.temp.path().join("go-calls"))
            .unwrap()
            .contains(f.temp.path().join("fresh-cache").to_str().unwrap())
    );
}

#[test]
fn update_forces_selected_builder_and_stops_on_failures() {
    let f = Fixture::new();
    f.tool(
        "git",
        r#"printf '%s:%s\n' "$PWD" "$*" >> "$CAPTURE/git-calls"
exit "${GIT_EXIT:-0}""#,
    );
    f.tool(
        "go",
        r#"printf '%s\n' "$*" >> "$CAPTURE/go-calls"
exit "${BUILD_EXIT:-0}""#,
    );
    f.tool(
        "docker",
        r#"printf '%s\n' "$*" >> "$CAPTURE/docker-calls"
exit "${BUILD_EXIT:-0}""#,
    );
    success(f.run(&["update"]));
    success(f.run(&["update", "--build-target", "host"]));
    assert_eq!(
        fs::read_to_string(f.temp.path().join("go-calls"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    let git = fs::read_to_string(f.temp.path().join("git-calls")).unwrap();
    assert_eq!(git.lines().count(), 4);
    assert!(git.lines().all(|line| line.ends_with("pull --ff-only -q")));
    success(f.run(&["update", "--build-target", "linux"]));
    assert!(f.temp.path().join("docker-calls").exists());
    assert_eq!(
        fs::read_to_string(f.cache.join("courses/demo/bin/OS")).unwrap(),
        "linux"
    );
    let go_before = fs::read(f.temp.path().join("go-calls")).unwrap();
    let output = f
        .command(&f.work)
        .env("GIT_EXIT", "17")
        .arg("update")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(f.temp.path().join("go-calls")).unwrap(), go_before);
    assert!(!stdout(&output).contains("updated course and tester"));
    let output = f
        .command(&f.work)
        .env("BUILD_EXIT", "18")
        .args(["update", "--build-target", "host"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!stdout(&output).contains("updated course and tester"));
    assert_eq!(
        fs::read_to_string(f.cache.join("courses/demo/bin/OS")).unwrap(),
        "linux"
    );
}

#[test]
fn update_help_accepts_only_host_and_linux() {
    let f = Fixture::new();
    let output = success(f.run(&["update", "--help"]));
    assert!(stdout(&output).contains("--build-target"));
    assert!(stdout(&output).contains("host"));
    assert!(stdout(&output).contains("linux"));
    for value in ["auto", "windows", "bogus"] {
        assert_eq!(
            f.run(&["update", "--build-target", value]).status.code(),
            Some(2)
        );
    }
}

#[test]
fn extension_add_is_idempotent_and_remove_retains_progress_and_saved_order() {
    let f = Fixture::new();
    success(f.run(&["ext", "add", "beta"]));
    success(f.run(&["ext", "add", "alpha"]));
    let before = f.config_bytes();
    success(f.run(&["ext", "add", "alpha"]));
    assert_eq!(f.config_bytes(), before);
    assert_eq!(f.config()["extensions"], json!(["beta", "alpha"]));
    success(f.run(&["submit", "--upto", "extra-a"]));
    assert_eq!(f.config()["completed"], json!(["extra-a"]));
    let original = success(f.run(&["show", "5", "--json"]));
    let original: Value = serde_json::from_slice(&original.stdout).unwrap();
    assert_eq!(original["slug"], "extra-a");
    assert_eq!(original["completed"], true);
    success(f.run(&["ext", "remove", "alpha"]));
    assert_eq!(f.config()["completed"], json!(["extra-a"]));
    assert!(!f.run(&["show", "extra-a", "--json"]).status.success());
    success(f.run(&["ext", "add", "alpha"]));
    let restored = success(f.run(&["show", "5", "--json"]));
    let restored: Value = serde_json::from_slice(&restored.stdout).unwrap();
    assert_eq!(restored, original);
    let status = stdout(&success(f.run(&["status"])));
    assert!(status.find("Extra B").unwrap() < status.find("Extra A").unwrap());
}
