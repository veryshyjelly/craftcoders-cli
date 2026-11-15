# Design

## Context

See proposal.md for motivation. The isolated audit at `/tmp/craftcoders-audit-hjlgvpco/results.json` reproduced failures with cached synthetic courses and fake tools. Source confirms `tester::run_tester` copies all environment variables into a debug-printable Command and slices its preview at byte 120; `config::find_project_root` starts with relative `.`; `cmd_submit` duplicates test orchestration and compiles before checking dry-run; `cmd_update` permits matching-marker cache reuse. `cmd_init` checks its destination after acquisition/build. Main's owo-colors override does not disable unconditional styling. Existing baseline requirements already promise nested workspace discovery and correct Linux fallback markers; these fixes restore those contracts, while deltas describe intentional behavior changes.

## Goals / Non-Goals

**Goals:** Keep changes small, isolate side effects after validation, share test/submission preparation, and establish executable behavioral coverage around real CLI subprocesses.

**Non-Goals:** No new workspace schema, stage-order changes, hosted dependencies, language-template rewrite, dependency cleanup, atomic persistence/locking redesign, or all-prefix submission completion. Do not edit `.serena/`, generated `.agents/` skills, or unrelated user changes.

## Decisions

### Stable cross-owner interfaces

- Tester owner exports `pub enum BuildTarget { Auto, Host, Linux }` and `pub fn build_tester(course_paths: &CoursePaths, target: BuildTarget, force: bool) -> Result<()>` from `tester.rs`. The obsolete course argument is removed. Derive Copy/Clone as useful. Auto resolves to host with Go, otherwise Linux via Docker, before cache comparison; Host requires Go; Linux always selects Docker. Cache reuse is allowed only when `force == false`; markers represent the actual successful target.
- Workflow owner defines `UpdateArgs` and a distinct optional CLI enum with Host/Linux values in `cli.rs`; use `Commands::Update(UpdateArgs)`. `cmd_update(args: &UpdateArgs, cli_cc_home: Option<&Path>, no_color: bool) -> Result<()>` maps the option to tester BuildTarget, defaults Auto, and passes `force=true` after successful pulls. Init passes Auto,false. Output owner adapts main's dispatch to `Commands::Update(args) => cmd_update(&args, ...)`.
- Preserve `run_tester(project_root: &Path, tester_binary: &Path, test_cases: &[TestCase], docker_image: Option<&str>, dry_run: bool) -> Result<i32>`. Tester owner resolves existing workspace/tester paths before setting the child cwd; host cwd is workspace root and Docker remains `/app`. CLI cache resolution returns absolute paths, resolving relative values against invocation cwd without requiring an existing directory. Avoid changing the parent process cwd.

### Shared preparation with controlled side effects

Workflow owner extracts bounded private preparation/execution helpers used by test and submit; no public abstraction or new module is required. Resolve configuration, ordered stages, and target before compilation. Explicit `test --upto` can select any enabled stage after all stages are complete. A submit still requires an unfinished current stage, preserving its previous all-complete behavior. Dry-run bypasses compilation and persistence in both commands; compile failure stops execution and retains saved state. Preserve tester exit statuses and target-only successful submission. Recompute the next current stage from updated completion instead of target index. Avoid duplicate saved target entries when submitting an already completed explicit target while other stages remain.

Alternative: patch duplicated branches independently. Rejected because the existing submit dry-run divergence demonstrates the cost of duplicated ordering logic. Alternative: mark every tested stage complete. Rejected by the owner's explicit target-only decision.

### Safe process descriptions

Use normal Command environment inheritance and apply only the three official overrides explicitly. Preview only intentional executable/arguments/contract information; never enumerate unrelated environment entries. Prefer full output over arbitrary truncation, or truncate at character boundaries. Preserve inherited child environment and stdio. Resolve absolute mount paths for relative cache input. Error guidance for incompatible Docker markers names `update --build-target linux`; host guidance names `update --build-target host`.

Alternative: filter a denylist of secret variable names. Rejected because new secret names would still leak. Clearing the execution environment is also rejected because it changes the tester compatibility contract; only tests clear ambient credentials.

### Central terminal output policy

Output owner may add `src/output.rs` and lexical print/eprint wrappers before module declarations in main, backed by existing anstream, to strip ANSI for disabled color without touching all styled call sites. Configure policy before command output, handle parser-owned output as appropriate, and ensure tracing does not emit ANSI when disabled. This covers CLI-owned output, not inherited output from compile/tester/editor subprocesses. Test stdout and stderr, including early errors, Markdown, and successful commands. Use nonempty NO_COLOR and explicit flag; retaining stronger suppression for an empty variable is allowed. Use current fetched library documentation when selecting exact APIs.

Alternative: replace every owo-colors styling call. Rejected because it creates broad overlapping edits across command, tester, and Git modules and easily misses call sites. The existing anstream dependency avoids adding a dependency solely for output policy.

### Isolated tests and ownership

- Workflow owner: `src/commands.rs`, `src/config.rs`, `src/cli.rs`, `tests/workflows.rs`.
- Tester owner: `src/tester.rs`, `tests/tester.rs`.
- Output/documentation owner: `src/main.rs`, new `src/output.rs`, `src/render.rs` only if central handling needs it, `tests/output.rs`, `README.md`, and narrowly related `openspec/README.md` guidance.
- Root integrates shared helpers only if useful (`tests/common/mod.rs`), owns final artifact task updates and any explicitly necessary manifest changes. No dependency change is expected. Independent tests can use standalone helpers to avoid a shared-file blocker.

Each integration test uses a unique temporary workspace/cache and a spawned CLI process, with fake executable scripts for Git/Go/Docker/tester/compile. Clear subprocess environments, add only known PATH/home/cache/test variables, and use synthetic secrets. Do not mutate process-global cwd/env in parallel Rust tests. Shell fixtures may be Unix-gated with that limitation reported. Never download real courses or run real Go/Docker builds in tests. Use direct per-file rustfmt while other owners are editing; root runs whole-repository checks after integration.

### Evidence-based deferrals

`project::save_project` uses direct `fs::write`, so interruption/concurrent-write vulnerability is plausible. The audit did not reproduce data loss, and atomic replacement does not itself solve concurrent lost updates. Defer persistence semantics to a separate change with a defined concurrency contract. A synthetic nested same-language template produces malformed output, but a read-only scan of the existing Redis definition corpus found 214 candidate Markdown/YAML files, 14 language template tags, and no nested same-language openings. No actual supported course fixture establishes that nesting is required; retain parser behavior and record the limitation rather than rewriting it. Apparently unused dependencies do not cause the reproduced failures and textual references do not establish measured gains; defer churn. README must stop claiming unsupported flags, language config, automatic all-prefix completion, or an unverified Rust 1.70 minimum; edition 2024 rules out that claim. Document a verified toolchain or omit an exact minimum instead of guessing dependency-wide MSRV.

## Risks / Trade-offs

- Forced updates take longer and may require build dependencies even with a cached binary → this is deliberate freshness behavior; report build errors instead of success.
- Existing builds write directly into shared cache output → do not promise transactional rollback or concurrent build safety; update marker only after successful build.
- A command's relative cache path is invocation-relative → document and test it so nested invocation cannot silently redirect child paths.
- Central wrappers can miss library output or broken-pipe handling → inspect remaining direct writes and cover stderr, parser paths, and rendered Markdown; use library-native color configuration where necessary.
- Unix shell integration fixtures do not demonstrate Windows execution → report actual tested platforms rather than claiming cross-platform verification.

## Migration Plan

No stored-data migration is required. Existing `.craftcoders.json` and cache layout stay readable. Deploy as a CLI update; use the explicit build-target option to recover the required tester platform. Rollback restores prior executable behavior but does not require rewriting workspace files. Keep this change active until implementation and independent review finish; do not archive or sync baseline specs prematurely.
