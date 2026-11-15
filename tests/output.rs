#![cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    home: PathBuf,
    tools: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let home = temp.path().join("cache");
        let tools = temp.path().join("tools");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&tools).unwrap();
        let definition = home.join("courses/demo/definition");
        fs::create_dir_all(definition.join(".git")).unwrap();
        fs::create_dir_all(home.join("courses/demo/tester/.git")).unwrap();
        fs::create_dir_all(home.join("courses/demo/bin")).unwrap();
        fs::write(
            root.join(".craftcoders.json"),
            r#"{"course":"demo","extensions":[],"completed":[]}"#,
        )
        .unwrap();
        fs::write(definition.join("course-definition.yml"), "name: Demo\nslug: demo\nstages:\n  - name: First\n    slug: first\n    difficulty: easy\n    marketing_md: '# Hello **world**'\n").unwrap();
        fs::write(home.join("courses/demo/bin/OS"), std::env::consts::OS).unwrap();
        fs::write(home.join("courses/demo/bin/tester"), "#!/bin/sh\nexit 0\n").unwrap();
        for tool in ["git", "go", "docker"] {
            let path = tools.join(tool);
            fs::write(&path, "#!/bin/sh\nif [ \"$1\" = build ]; then\n  while [ \"$#\" -gt 0 ]; do\n    if [ \"$1\" = -o ]; then shift; printf '#!/bin/sh\\nexit 0\\n' > \"$1\"; break; fi\n    shift\n  done\nfi\nexit 0\n").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self {
            _temp: temp,
            root,
            home,
            tools,
        }
    }
    fn run(&self, args: &[&str], env_color: bool) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_craftcoders"));
        command
            .current_dir(&self.root)
            .env_clear()
            .env("HOME", &self.root)
            .env("CC_HOME", &self.home)
            .env("PATH", &self.tools)
            .env("TERM", "xterm-256color")
            .env("CLICOLOR_FORCE", "1");
        if env_color {
            command.env("NO_COLOR", "1");
        } else {
            command.arg("--no-color");
        }
        command.args(args).output().unwrap()
    }
}
fn plain(output: &Output) {
    for stream in [&output.stdout, &output.stderr] {
        assert!(
            !stream.contains(&0x1b),
            "ANSI output: {}",
            String::from_utf8_lossy(stream)
        );
    }
}
#[test]
fn cli_owned_output_honors_flag_and_environment() {
    for env_color in [false, true] {
        let fixture = Fixture::new();
        for args in [
            vec!["status"],
            vec!["show"],
            vec!["test", "--dry-run"],
            vec!["submit", "--dry-run"],
            vec!["doctor"],
            vec!["list"],
            vec!["init", "demo", "new-project", "--no-build"],
            vec!["update", "--build-target", "host"],
        ] {
            let output = fixture.run(&args, env_color);
            plain(&output);
            assert!(
                output.status.success(),
                "{args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        for args in [
            vec!["show", "missing"],
            vec!["test", "--upto", "999"],
            vec!["unknown-command"],
            vec!["update", "--build-target", "invalid"],
        ] {
            let output = fixture.run(&args, env_color);
            plain(&output);
            assert!(!output.status.success(), "{args:?}");
        }
        let help = fixture.run(&["--help"], env_color);
        plain(&help);
        assert!(help.status.success());
    }
}
#[test]
fn raw_markdown_and_json_keep_their_formats() {
    for env_color in [false, true] {
        let fixture = Fixture::new();
        let raw = fixture.run(&["show", "--raw"], env_color);
        plain(&raw);
        assert!(raw.status.success());
        assert_eq!(
            String::from_utf8(raw.stdout).unwrap(),
            "# Hello **world**\n"
        );
        let json = fixture.run(&["show", "--json"], env_color);
        plain(&json);
        assert!(json.status.success());
        let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
        assert_eq!(value["slug"], "first");
        assert_eq!(value["markdown"], "# Hello **world**\n");
    }
}
