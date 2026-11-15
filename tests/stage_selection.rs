use serde_json::{Value, json};
use std::fs;
use std::process::{Command, Output};
use tempfile::{TempDir, tempdir};

struct Project {
    root: TempDir,
}

impl Project {
    fn new() -> Self {
        let root = tempdir().unwrap();
        let definition = root.path().join("courses/example/definition");
        fs::create_dir_all(&definition).unwrap();
        fs::write(
            definition.join("course-definition.yml"),
            "name: Example\nslug: example\nstages:\n  - {name: First, slug: first}\n  - {name: Second, slug: second}\n  - {name: Third, slug: third}\n",
        ).unwrap();
        fs::write(
            root.path().join(".craftcoders.json"),
            json!({"course": "example", "completed": ["first"]}).to_string(),
        )
        .unwrap();
        Self { root }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_craftcoders"))
            .args(args)
            .arg("--cc-home")
            .arg(self.root.path())
            .arg("--no-color")
            .current_dir(self.root.path())
            .output()
            .unwrap()
    }

    fn completed(&self) -> Value {
        let config = fs::read(self.root.path().join(".craftcoders.json")).unwrap();
        serde_json::from_slice::<Value>(&config).unwrap()["completed"].clone()
    }

    fn dry_run_slugs(&self, args: &[&str]) -> Vec<String> {
        let output = self.run(args);
        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let (_, cases) = stdout.split_once("test cases:\n").unwrap();
        slugs(serde_json::from_str(cases).unwrap())
    }

    #[cfg(unix)]
    fn install_tester(&self, exit_code: i32) {
        use std::os::unix::fs::PermissionsExt;

        let bin = self.root.path().join("courses/example/bin");
        fs::create_dir_all(&bin).unwrap();
        let tester = bin.join("tester");
        fs::write(&tester, format!(
            "#!/bin/sh\nprintf '%s' \"$CODECRAFTERS_TEST_CASES_JSON\" > \"$CODECRAFTERS_REPOSITORY_DIR/test-cases.json\"\nexit {exit_code}\n"
        )).unwrap();
        fs::set_permissions(tester, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    fn executed_slugs(&self) -> Vec<String> {
        let cases = fs::read(self.root.path().join("test-cases.json")).unwrap();
        slugs(serde_json::from_slice(&cases).unwrap())
    }
}

fn slugs(cases: Value) -> Vec<String> {
    cases
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["slug"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn test_defaults_to_current_stage() {
    let project = Project::new();
    assert_eq!(project.dry_run_slugs(&["test", "--dry-run"]), ["second"]);
    assert_eq!(project.completed(), json!(["first"]));
    assert_eq!(
        project.dry_run_slugs(&["test", "--only-current", "--dry-run"]),
        ["second"]
    );
}

#[test]
fn explicit_upto_preserves_range_and_only_current_override() {
    let project = Project::new();
    for stage in ["3", "third"] {
        assert_eq!(
            project.dry_run_slugs(&["test", "--upto", stage, "--dry-run"]),
            ["first", "second", "third"]
        );
        assert_eq!(
            project.dry_run_slugs(&["test", "--upto", stage, "--only-current", "--dry-run"]),
            ["third"]
        );
    }
    assert_eq!(project.completed(), json!(["first"]));
}

#[test]
fn submit_dry_run_includes_previous_stages_without_advancing() {
    let project = Project::new();
    assert_eq!(
        project.dry_run_slugs(&["submit", "--dry-run"]),
        ["first", "second"]
    );
    assert_eq!(project.completed(), json!(["first"]));
}

#[cfg(unix)]
#[test]
fn test_runs_current_stage_without_advancing() {
    let project = Project::new();
    project.install_tester(0);
    let output = project.run(&["test"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(project.executed_slugs(), ["second"]);
    assert_eq!(project.completed(), json!(["first"]));
}

#[cfg(unix)]
#[test]
fn submit_checks_previous_stages_and_advances_only_on_success() {
    for exit_code in [0, 23] {
        let project = Project::new();
        project.install_tester(exit_code);
        let output = project.run(&["submit"]);
        assert_eq!(output.status.code(), Some(exit_code), "{output:?}");
        assert_eq!(project.executed_slugs(), ["first", "second"]);
        let expected = if exit_code == 0 {
            json!(["first", "second"])
        } else {
            json!(["first"])
        };
        assert_eq!(project.completed(), expected);
    }
}
