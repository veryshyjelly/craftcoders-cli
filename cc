#!/usr/bin/env python3
"""cc - practice CodeCrafters-style challenges locally.

Uses the public course repos (stage text + stage list) and the official
testers from github.com/codecrafters-io. No starters, no solutions, no website:
you write the program in any language, `cc` shows what each stage asks for and
runs the real tester against your code.

    cc init redis my-redis     # fetch course + tester, build tester, make project
    cd my-redis
    cc show                    # what the current stage asks you to build
    cc test                    # run previous stages + current stage
    cc submit                  # same, and advance to the next stage on a pass
    cc status                  # progress
    cc ext add lists           # opt in to an extension's stages
    cc update                  # pull latest course + tester and rebuild
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("cc needs PyYAML:  pip install pyyaml")

ORG = "https://github.com/codecrafters-io"
CC_HOME = Path(os.environ.get("CC_HOME", Path.home() / ".cc"))
PROJECT_FILE = ".cc.json"

TTY = sys.stdout.isatty()


def c(code, text):
    return f"\033[{code}m{text}\033[0m" if TTY else text


def die(msg):
    sys.exit(c("31", "error: ") + msg)


# --------------------------------------------------------------------------
# Course + tester storage
# --------------------------------------------------------------------------

def course_dir(course):
    return CC_HOME / "courses" / course


def definition_dir(course):
    return course_dir(course) / "definition"


def tester_dir(course):
    return course_dir(course) / "tester"


def tester_bin(course):
    return course_dir(course) / "bin" / "tester"


def git_clone(url, dest):
    dest.parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(["git", "clone", "--depth", "1", url, str(dest)])
    if r.returncode != 0:
        die(f"could not clone {url}")


def ensure_repos(course, tester_repo=None):
    if not definition_dir(course).exists():
        git_clone(f"{ORG}/build-your-own-{course}.git", definition_dir(course))
    if not tester_dir(course).exists():
        git_clone(f"{ORG}/{tester_repo or course + '-tester'}.git", tester_dir(course))


def build_tester(course):
    """Compile the official tester. Prefers local Go, falls back to Docker."""
    tdir = tester_dir(course)
    out = tester_bin(course)
    out.parent.mkdir(parents=True, exist_ok=True)
    pkg = "./cmd/tester" if (tdir / "cmd" / "tester").is_dir() else "."
    marker = out.parent / "OS"

    if shutil.which("go"):
        print(c("2", f"building tester with go ({pkg}) ..."))
        r = subprocess.run(["go", "build", "-o", str(out), pkg], cwd=tdir)
        marker.write_text(sys.platform)
    elif shutil.which("docker"):
        print(c("2", "go not found; building tester inside docker (linux binary) ..."))
        r = subprocess.run([
            "docker", "run", "--rm",
            "-v", f"{tdir}:/src", "-v", f"{out.parent}:/out", "-w", "/src",
            "-e", "CGO_ENABLED=0", "golang:1.24-bookworm",
            "go", "build", "-o", "/out/tester", pkg,
        ])
        marker.write_text("linux")
    else:
        die("need either Go or Docker to build the tester")
    if r.returncode != 0 or not out.exists():
        die("tester build failed (see output above)")


def load_definition(course):
    path = definition_dir(course) / "course-definition.yml"
    if not path.exists():
        die(f"course '{course}' is not installed; run: cc init {course}")
    with open(path) as f:
        return yaml.safe_load(f)


def render_markdown(text):
    """Resolve the site's template blocks for a human reader with no fixed language."""
    text = re.sub(r"\{\{#reader_is_bot\}\}.*?\{\{/reader_is_bot\}\}", "", text, flags=re.S)
    text = re.sub(r"\{\{#reader_is_human\}\}(.*?)\{\{/reader_is_human\}\}", r"\1", text, flags=re.S)
    # {{#lang_is_x}} blocks only apply to one language: drop them.
    text = re.sub(r"\{\{#lang_is_(\w+)\}\}.*?\{\{/lang_is_\1\}\}", "", text, flags=re.S)
    # {{^lang_is_x}} blocks apply to every other language: keep them.
    # They can be nested, so repeat until nothing changes.
    while True:
        new = re.sub(r"\{\{\^lang_is_(\w+)\}\}(.*?)\{\{/lang_is_\1\}\}", r"\2", text, flags=re.S)
        if new == text:
            break
        text = new
    text = text.replace("{{language_name}}", "your language")
    return re.sub(r"\n{3,}", "\n\n", text).strip() + "\n"


def stage_text(course, slug):
    d = definition_dir(course) / "stage_descriptions"
    for p in sorted(d.glob(f"*-{slug}.md")):
        return render_markdown(p.read_text())
    return None


def ordered_stages(defn, enabled):
    stages = defn["stages"]
    out = [s for s in stages if not s.get("primary_extension_slug")]
    for ext in defn.get("extensions", []):
        if ext["slug"] in enabled:
            out += [s for s in stages if s.get("primary_extension_slug") == ext["slug"]]
    return out


# --------------------------------------------------------------------------
# Project state
# --------------------------------------------------------------------------

def find_project():
    p = Path.cwd().resolve()
    for d in [p, *p.parents]:
        if (d / PROJECT_FILE).exists():
            return d
    die("not inside a cc project (no .cc.json found). Run `cc init <course>` first.")


def load_project():
    root = find_project()
    cfg = json.loads((root / PROJECT_FILE).read_text())
    defn = load_definition(cfg["course"])
    return root, cfg, defn


def save_project(root, cfg):
    (root / PROJECT_FILE).write_text(json.dumps(cfg, indent=2) + "\n")


def current_index(stages, cfg):
    done = set(cfg["completed"])
    for i, s in enumerate(stages):
        if s["slug"] not in done:
            return i
    return None  # everything complete


# --------------------------------------------------------------------------
# Commands
# --------------------------------------------------------------------------

def cmd_init(a):
    course = a.course
    ensure_repos(course, a.tester_repo)
    if not tester_bin(course).exists():
        build_tester(course)

    target = Path(a.dir or f"my-{course}").resolve()
    if target.exists() and any(target.iterdir()):
        die(f"{target} already exists and is not empty")
    target.mkdir(parents=True, exist_ok=True)

    defn = load_definition(course)
    save_project(target, {"course": course, "extensions": [], "completed": []})
    (target / "codecrafters.yml").write_text(
        "# Read by the tester. Set debug to true for verbose logs.\n"
        "debug: false\n"
    )
    entry = target / "your_program.sh"
    entry.write_text(
        "#!/bin/sh\n"
        "# The tester runs this file to start your program.\n"
        "# Replace the line below with whatever runs your implementation, e.g.\n"
        "#   exec python3 -u app/main.py \"$@\"\n"
        "#   exec go run ./app \"$@\"\n"
        "echo \"edit your_program.sh to start your program\" >&2\n"
        "exit 1\n"
    )
    entry.chmod(0o755)
    if shutil.which("git"):
        subprocess.run(["git", "init", "-q"], cwd=target)

    print(c("32", f"Created {target}"))
    print(f"{defn['name']}: {len(ordered_stages(defn, []))} base stages.")
    print(f"Next:  cd {target} && cc show")


def cmd_status(a):
    root, cfg, defn = load_project()
    stages = ordered_stages(defn, cfg["extensions"])
    cur = current_index(stages, cfg)
    print(c("1", defn["name"]) + f"   ({len(cfg['completed'])}/{len(stages)} done)")
    for i, s in enumerate(stages):
        if s["slug"] in cfg["completed"]:
            mark = c("32", "✔")
        elif i == cur:
            mark = c("33", "▶")
        else:
            mark = c("2", "·")
        ext = f"  [{s['primary_extension_slug']}]" if s.get("primary_extension_slug") else ""
        print(f" {mark} {i + 1:>3}. {s['name']}{c('2', ext)}")
    if cur is None:
        print(c("32", "All stages complete."))


def pick_stage(stages, cfg, which):
    if which is None:
        cur = current_index(stages, cfg)
        if cur is None:
            die("all stages are complete. Enable more with `cc ext add <slug>`.")
        return cur
    if which.isdigit() and 1 <= int(which) <= len(stages):
        return int(which) - 1
    for i, s in enumerate(stages):
        if s["slug"] == which:
            return i
    die(f"no stage '{which}' (use a number 1-{len(stages)} or a slug)")


def cmd_show(a):
    root, cfg, defn = load_project()
    stages = ordered_stages(defn, cfg["extensions"])
    i = pick_stage(stages, cfg, a.stage)
    s = stages[i]
    done = " (done)" if s["slug"] in cfg["completed"] else ""
    head = f"Stage {i + 1}/{len(stages)} · {s['name']} · {s.get('difficulty', '?')}{done}"
    print(c("1", head))
    print(c("2", "─" * len(head)))
    body = stage_text(cfg["course"], s["slug"])
    print(body if body else s.get("marketing_md", "(no description found)"))
    print(c("2", "\nWhen you think it works:  cc test   |   to advance:  cc submit"))


def build_test_cases(stages, upto, only_current):
    idxs = [upto] if only_current else list(range(upto + 1))
    return [
        {
            "slug": stages[i]["slug"],
            "tester_log_prefix": f"stage-{i + 1}",
            "title": f"Stage #{i + 1}: {stages[i]['name']}",
        }
        for i in idxs
    ]


def run_tester(root, cfg, cases, docker_image, dry_run):
    course = cfg["course"]
    binary = tester_bin(course)
    if not binary.exists():
        die("tester binary missing; run `cc update`")
    env_vars = {
        "CODECRAFTERS_REPOSITORY_DIR": str(root),
        "CODECRAFTERS_SUBMISSION_DIR": str(root),  # older testers use this name
        "CODECRAFTERS_TEST_CASES_JSON": json.dumps(cases),
    }

    if docker_image:
        env_vars["CODECRAFTERS_REPOSITORY_DIR"] = "/app"
        env_vars["CODECRAFTERS_SUBMISSION_DIR"] = "/app"
        cmd = ["docker", "run", "--rm", "-i",
               "-v", f"{root}:/app", "-v", f"{binary}:/usr/local/bin/tester:ro",
               "-w", "/app"]
        for k, v in env_vars.items():
            cmd += ["-e", f"{k}={v}"]
        cmd += [docker_image, "tester"]
        env = None
        if (binary.parent / "OS").exists() and (binary.parent / "OS").read_text() != "linux":
            die("docker mode needs a linux tester binary; rebuild it on linux, or "
                "build with GOOS=linux CGO_ENABLED=0 go build ... into "
                f"{binary}")
    else:
        if (binary.parent / "OS").exists() and (binary.parent / "OS").read_text() == "linux" \
                and sys.platform != "linux":
            die("tester was built for linux (via docker); use --docker IMAGE, or install Go "
                "and run `cc update`")
        cmd = [str(binary)]
        env = {**os.environ, **env_vars}

    if dry_run:
        print("command:", " ".join(cmd[:8]) + (" ..." if len(cmd) > 8 else ""))
        print("test cases:\n" + json.dumps(cases, indent=2))
        return 0
    return subprocess.run(cmd, cwd=root, env=env).returncode


def run_for_current(a, only_current):
    root, cfg, defn = load_project()
    stages = ordered_stages(defn, cfg["extensions"])
    cur = current_index(stages, cfg)
    if cur is None:
        die("all stages are complete. Enable more with `cc ext add <slug>`.")
    cases = build_test_cases(stages, cur, only_current)
    print(c("1", f"Running {len(cases)} stage(s) up to: {stages[cur]['name']}\n"))
    rc = run_tester(root, cfg, cases, a.docker, a.dry_run)
    return root, cfg, stages, cur, rc


def cmd_test(a):
    _, _, _, _, rc = run_for_current(a, a.only_current)
    sys.exit(rc)


def cmd_submit(a):
    root, cfg, stages, cur, rc = run_for_current(a, False)
    if a.dry_run:
        return
    if rc != 0:
        print(c("31", "\nNot passing yet. Keep going, then run `cc submit` again."))
        sys.exit(rc)
    cfg["completed"].append(stages[cur]["slug"])
    save_project(root, cfg)
    print(c("32", f"\n✔ Stage {cur + 1} complete: {stages[cur]['name']}"))
    if cur + 1 < len(stages):
        print(f"Next: stage {cur + 2} - {stages[cur + 1]['name']}   (run `cc show`)")
    else:
        print(c("32", "That was the last enabled stage. Add more with `cc ext add <slug>`."))


def cmd_ext(a):
    root, cfg, defn = load_project()
    exts = {e["slug"]: e for e in defn.get("extensions", [])}
    if a.action == "list":
        for slug, e in exts.items():
            on = c("32", "on ") if slug in cfg["extensions"] else c("2", "off")
            n = sum(1 for s in defn["stages"] if s.get("primary_extension_slug") == slug)
            print(f" {on}  {slug:<20} {e['name']} ({n} stages)")
        return
    if not a.slug or a.slug not in exts:
        die(f"unknown extension; choose from: {', '.join(exts)}")
    if a.action == "add" and a.slug not in cfg["extensions"]:
        cfg["extensions"].append(a.slug)
    if a.action == "remove" and a.slug in cfg["extensions"]:
        cfg["extensions"].remove(a.slug)
    save_project(root, cfg)
    print(f"extensions enabled: {', '.join(cfg['extensions']) or '(none)'}")


def cmd_update(a):
    root, cfg, _ = load_project()
    course = cfg["course"]
    for d in (definition_dir(course), tester_dir(course)):
        subprocess.run(["git", "pull", "--ff-only", "-q"], cwd=d)
    build_tester(course)
    print(c("32", "updated course and tester"))


def main():
    p = argparse.ArgumentParser(prog="cc", description="Practice CodeCrafters challenges locally.")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("init", help="set up a course and create a project folder")
    s.add_argument("course", help="course slug, e.g. redis, git, sqlite, shell")
    s.add_argument("dir", nargs="?", help="project folder (default: my-<course>)")
    s.add_argument("--tester-repo", help="override tester repo name (default: <course>-tester)")
    s.set_defaults(fn=cmd_init)

    s = sub.add_parser("status", help="show progress")
    s.set_defaults(fn=cmd_status)

    s = sub.add_parser("show", help="show a stage's instructions (default: current)")
    s.add_argument("stage", nargs="?", help="stage number or slug")
    s.set_defaults(fn=cmd_show)

    for name, fn, helptext in (("test", cmd_test, "run previous stages + current stage"),
                               ("submit", cmd_submit, "run tests and advance on a pass")):
        s = sub.add_parser(name, help=helptext)
        s.add_argument("--docker", metavar="IMAGE",
                       help="run the tester inside this image instead of on the host")
        s.add_argument("--dry-run", action="store_true", help="print what would run")
        if name == "test":
            s.add_argument("--only-current", action="store_true",
                           help="run only the current stage")
        s.set_defaults(fn=fn)

    s = sub.add_parser("ext", help="list/enable/disable extensions")
    s.add_argument("action", nargs="?", default="list", choices=["list", "add", "remove"])
    s.add_argument("slug", nargs="?")
    s.set_defaults(fn=cmd_ext)

    s = sub.add_parser("update", help="pull latest course + tester and rebuild")
    s.set_defaults(fn=cmd_update)

    a = p.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
