#![cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
    workspace: PathBuf,
    course: PathBuf,
    tools: PathBuf,
}
fn executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "craftcoders-tester-{}-{}-界",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let workspace = root.join("workspace界");
        let course = root.join("cache界/courses/demo");
        let tools = root.join("tools");
        for path in [
            &workspace.join("nested/deeper"),
            &course.join("definition/stage_descriptions"),
            &course.join("tester/.git"),
            &course.join("definition/.git"),
            &course.join("bin"),
            &tools,
        ] {
            fs::create_dir_all(path).unwrap();
        }
        fs::write(course.join("definition/course-definition.yml"),"name: Demo\nslug: demo\nstages:\n  - {name: First, slug: one}\n  - {name: Second, slug: two}\n  - {name: Third界, slug: three}\n").unwrap();
        fs::write(
            workspace.join(".craftcoders.json"),
            r#"{"course":"demo","extensions":[],"completed":[]}"#,
        )
        .unwrap();
        executable(
            &workspace.join("compile.sh"),
            "#!/bin/sh\necho compiled >> \"$CAPTURE.compile\"\n",
        );
        executable(
            &course.join("bin/tester"),
            "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$CODECRAFTERS_REPOSITORY_DIR\" \"$CODECRAFTERS_SUBMISSION_DIR\" \"$SYNTHETIC_SECRET\" \"$CODECRAFTERS_TEST_CASES_JSON\" > \"$CAPTURE\"\nexit \"${TESTER_RC:-0}\"\n",
        );
        fs::write(course.join("bin/OS"), std::env::consts::OS).unwrap();
        executable(
            &tools.join("git"),
            "#!/bin/sh\necho git >> \"$TOOLS_CAPTURE\"\nexit 0\n",
        );
        executable(
            &tools.join("docker"),
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$TOOLS_CAPTURE\"\nexit \"${BUILD_RC:-0}\"\n",
        );
        Self {
            root,
            workspace,
            course,
            tools,
        }
    }
    fn go(&self) {
        executable(
            &self.tools.join("go"),
            "#!/bin/sh\necho native-go >> \"$TOOLS_CAPTURE\"\nexit \"${BUILD_RC:-0}\"\n",
        );
    }
    fn artifact_go(&self) {
        executable(
            &self.tools.join("go"),
            r#"#!/bin/sh
printf '%s\n' "$PWD" "$@" > "$TOOLS_CAPTURE"
[ "$1" = build ] && [ "$2" = -o ] || exit 90
printf 'fresh tester artifact\n' > "$3"
"#,
        );
    }
    fn command(&self, args: &[&str], nested: bool) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_craftcoders"));
        c.args(args)
            .current_dir(if nested {
                self.workspace.join("nested/deeper")
            } else {
                self.workspace.clone()
            })
            .env_clear()
            .env("PATH", &self.tools)
            .env("HOME", &self.root)
            .env("CC_HOME", self.root.join("cache界"))
            .env("NO_COLOR", "1")
            .env("CAPTURE", self.root.join("capture"))
            .env("TOOLS_CAPTURE", self.root.join("tools.log"))
            .env("SYNTHETIC_SECRET", "synthetic-secret-界")
            .env("CODECRAFTERS_REPOSITORY_DIR", "incorrect")
            .env("CODECRAFTERS_SUBMISSION_DIR", "incorrect");
        c
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command(args, false).output().unwrap()
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.root.join("tools.log")).unwrap_or_default()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn success(o: &Output) {
    assert!(
        o.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
}
#[test]
fn host_inheritance_nested_relative_cache_and_cases() {
    let f = Fixture::new();
    let o = f
        .command(&["test", "--upto", "3"], true)
        .env("CC_HOME", "../../../cache界")
        .output()
        .unwrap();
    success(&o);
    let text = fs::read_to_string(f.root.join("capture")).unwrap();
    let lines: Vec<_> = text.lines().collect();
    for line in &lines[..3] {
        assert_eq!(*line, f.workspace.canonicalize().unwrap().to_str().unwrap());
    }
    assert_eq!(lines[3], "synthetic-secret-界");
    let cases: serde_json::Value = serde_json::from_str(lines[4]).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 3);
    assert_eq!(cases[2]["slug"], "three");
    assert_eq!(cases[2]["tester_log_prefix"], "stage-3");
    assert_eq!(cases[2]["title"], "Stage #3: Third界");
}
#[test]
fn only_current_and_nonzero_status() {
    let f = Fixture::new();
    let o = f
        .command(&["test", "--upto", "3", "--only-current"], false)
        .env("TESTER_RC", "7")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(7));
    let text = fs::read_to_string(f.root.join("capture")).unwrap();
    let cases: serde_json::Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 1);
    assert_eq!(cases[0]["slug"], "three");
    assert_eq!(cases[0]["tester_log_prefix"], "stage-3");
}
#[test]
fn preview_unicode_secret_and_missing_binary() {
    let f = Fixture::new();
    fs::remove_file(f.course.join("bin/tester")).unwrap();
    for cmd in ["test", "submit"] {
        let o = f.run(&[cmd, "--dry-run", "--upto", "3"]);
        success(&o);
        let s = String::from_utf8(o.stdout).unwrap();
        assert!(s.contains("command:"));
        assert!(s.contains("Third界"));
        assert!(!s.contains("synthetic-secret"));
    }
    assert!(!f.root.join("capture").exists());
    assert!(!f.root.join("capture.compile").exists());
}
#[test]
fn docker_absolute_mounts_and_contract() {
    let f = Fixture::new();
    fs::write(f.course.join("bin/OS"), "linux").unwrap();
    success(&f.run(&["test", "--docker", "fake-image"]));
    let calls = f.calls();
    assert!(calls.contains(&format!(
        "{}:/app",
        f.workspace.canonicalize().unwrap().display()
    )));
    assert!(calls.contains(&format!(
            "{}:/usr/local/bin/tester:ro",
            f.course
                .join("bin/tester")
                .canonicalize()
                .unwrap()
                .display()
        )));
    assert!(calls.contains("CODECRAFTERS_REPOSITORY_DIR=/app"));
    assert!(calls.contains("CODECRAFTERS_SUBMISSION_DIR=/app"));
    assert!(calls.contains("-w\n/app"));
}
#[test]
fn incompatible_docker_marker_rejects_launch() {
    let f = Fixture::new();
    fs::write(f.course.join("bin/OS"), "non-linux").unwrap();
    let o = f.run(&["test", "--docker", "fake-image"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("update --build-target linux"));
    assert!(f.calls().is_empty());
}
#[test]
fn init_reuses_matching_host_cache() {
    let f = Fixture::new();
    f.go();
    let dest = f.root.join("new-project");
    success(&f.run(&["init", "demo", dest.to_str().unwrap()]));
    assert!(!f.calls().contains("native-go"));
}
#[test]
fn update_forces_matching_build() {
    let f = Fixture::new();
    f.go();
    success(&f.run(&["update"]));
    assert!(f.calls().contains("native-go"));
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        std::env::consts::OS
    );
}
#[test]
fn auto_without_go_marks_linux() {
    let f = Fixture::new();
    fs::write(f.course.join("bin/OS"), "incompatible").unwrap();
    success(&f.run(&["update"]));
    assert!(f.calls().contains("golang:1.24-bookworm"));
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        "linux"
    );
}
#[test]
fn explicit_linux_uses_docker_with_go_present() {
    let f = Fixture::new();
    f.go();
    success(&f.run(&["update", "--build-target", "linux"]));
    assert!(f.calls().contains("golang:1.24-bookworm"));
    assert!(!f.calls().lines().any(|l| l == "native-go"));
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        "linux"
    );
}
#[test]
fn explicit_host_requires_go() {
    let f = Fixture::new();
    let o = f.run(&["update", "--build-target", "host"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("requires Go"));
    assert!(!f.calls().contains("golang"));
}
#[test]
fn failure_preserves_marker_and_omits_success() {
    let f = Fixture::new();
    f.go();
    fs::write(f.course.join("bin/OS"), "old-target").unwrap();
    let o = f
        .command(&["update"], false)
        .env("BUILD_RC", "9")
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        "old-target"
    );
    assert!(!String::from_utf8_lossy(&o.stdout).contains("built successfully"));
}

#[test]
fn init_auto_reuses_resolved_linux_cache_without_go() {
    let f = Fixture::new();
    fs::write(f.course.join("bin/OS"), "linux").unwrap();
    let dest = f.root.join("new-project");
    success(&f.run(&["init", "demo", dest.to_str().unwrap()]));
    assert!(!f.calls().contains("golang"));
}

#[test]
fn failed_docker_build_does_not_create_marker() {
    let f = Fixture::new();
    fs::remove_file(f.course.join("bin/OS")).unwrap();
    let o = f
        .command(&["update"], false)
        .env("BUILD_RC", "8")
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(!f.course.join("bin/OS").exists());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("built successfully"));
}

#[test]
#[cfg(not(target_os = "linux"))]
fn incompatible_host_marker_rejects_launch() {
    let f = Fixture::new();
    fs::write(f.course.join("bin/OS"), "linux").unwrap();
    let o = f.run(&["test"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("update --build-target host"));
    assert!(!f.root.join("capture").exists());
}

#[test]
fn forced_update_replaces_existing_tester_artifact() {
    let f = Fixture::new();
    f.artifact_go();
    let binary = f.course.join("bin/tester");
    fs::write(&binary, "stale tester artifact\n").unwrap();
    success(&f.run(&["update", "--build-target", "host"]));
    assert_eq!(
        fs::read_to_string(binary).unwrap(),
        "fresh tester artifact\n"
    );
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        std::env::consts::OS
    );
}

#[test]
fn fresh_host_build_creates_artifact_from_cmd_tester_package() {
    let f = Fixture::new();
    f.artifact_go();
    fs::create_dir_all(f.course.join("tester/cmd/tester")).unwrap();
    let binary = f.course.join("bin/tester");
    fs::remove_file(&binary).unwrap();
    fs::remove_file(f.course.join("bin/OS")).unwrap();
    let destination = f.root.join("fresh-project");
    success(&f.run(&["init", "demo", destination.to_str().unwrap()]));
    assert_eq!(
        fs::read_to_string(&binary).unwrap(),
        "fresh tester artifact\n"
    );
    assert_eq!(
        fs::read_to_string(f.course.join("bin/OS")).unwrap(),
        std::env::consts::OS
    );
    let calls = f.calls();
    let lines: Vec<_> = calls.lines().collect();
    assert_eq!(
        lines[0],
        f.course
            .join("tester")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(lines[1], "build");
    assert_eq!(lines[2], "-o");
    assert!(Path::new(lines[3]).is_absolute());
    assert_eq!(
        Path::new(lines[3]).canonicalize().unwrap(),
        binary.canonicalize().unwrap()
    );
    assert_eq!(lines[4], "./cmd/tester");
}

#[test]
fn successful_builder_without_output_fails_without_marker() {
    let f = Fixture::new();
    f.go();
    fs::remove_file(f.course.join("bin/tester")).unwrap();
    fs::remove_file(f.course.join("bin/OS")).unwrap();
    let o = f.run(&["update", "--build-target", "host"]);
    assert!(!o.status.success());
    assert!(f.calls().contains("native-go"));
    assert!(String::from_utf8_lossy(&o.stderr).contains("tester binary not found"));
    assert!(!f.course.join("bin/OS").exists());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("built successfully"));
}
