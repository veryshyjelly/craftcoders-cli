# Tasks

Implementation groups 1–3 may run in parallel with the file ownership and API contract in design.md. Owners report task IDs and evidence; root alone updates these checkboxes. Tests use isolated temporary paths and fake tools, never real downloads or user-cache writes. Each owner documents changed behavior in code/help as applicable; group 3 reconciles the shared user guide after reading those interfaces.

## 1. Workflow owner — commands, config, CLI, workflow tests

- [x] 1.1 Resolve workspace traversal from absolute start paths and resolve relative cache inputs against invocation cwd; verify nested status/show/test/submit and CLI-over-environment cache precedence using isolated integration tests, including a relative cache with a nonexistent initial path.
- [x] 1.2 Share bounded test/submit preparation and execution, select targets before compile, and bypass compile for both previews; verify compile failure prevents tester/progress writes, dry-run compile/tester markers remain absent with unchanged configuration bytes, and invalid selectors cause no compile side effect.
- [x] 1.3 Preserve tester exit codes and target-only submission, allow explicit test retests after all-complete, deduplicate repeated submission targets, and compute guidance from actual remaining progress; verify pass/fail test and submit cases, a gap after `submit --upto 3`, all-complete explicit test success, and no-target all-complete failure.
- [x] 1.4 Move destination validation before repository acquisition/build; verify nonempty directories and file destinations fail without fake Git/Go/Docker invocation or changed destination files, and empty/absent destinations still initialize with `--no-build`.
- [x] 1.5 Add `UpdateArgs`/`--build-target host|linux`, map to agreed BuildTarget API, and force rebuild after successful pulls; verify argument help/rejection and fake-tool calls show rebuild despite a matching marker, while pull failure prevents build and build failure prevents success output.
- [x] 1.6 Add extension compatibility regressions alongside workflow tests; verify add is idempotent, remove retains completed extension slugs, and re-enable restores the same completed status and ordering. Run `cargo test --test workflows` once cross-module interfaces compile and report results.

## 2. Tester owner — tester module and tester tests

- [x] 2.1 Implement BuildTarget Auto/Host/Linux and force semantics with markers based on the resolved successful target; verify fake Go/Docker builds cover init cache reuse, forced update rebuild, Auto fallback without Go, explicit Linux with Go present, explicit Host without Go, and builder failure without a newly successful marker. Document the API contract in code.
- [x] 2.2 Remove inherited-environment copying from command construction and produce a Unicode-safe preview; verify synthetic unrelated secrets are absent, Unicode workspace/cache paths do not panic, and host child environment inheritance plus official overrides remain intact.
- [x] 2.3 Run host testers from absolute workspace root and use absolute binary/mount paths; verify a nested invocation with relative cache finds the tester and supplies root cwd/official variables, while Docker receives `/app` and absolute mounts. Update incompatible-marker guidance to the explicit update target and test rejection without process launch.
- [x] 2.4 Add standalone isolated tester fixtures and run `cargo test --test tester`; verify testcase ordering/only-current payloads, preview output without execution, and nonzero tester exit behavior using captured files rather than real external tools.

## 3. Output/documentation owner — main, output policy, output tests, README

- [x] 3.1 Integrate the UpdateArgs dispatch and implement central CLI-owned color suppression with existing dependencies, using render.rs only where necessary; verify status, show Markdown, previews, errors, and diagnostics omit ANSI on stdout and stderr for `--no-color` and nonempty `NO_COLOR`, including early argument errors as applicable.
- [x] 3.2 Add `tests/output.rs` with isolated fixtures and run `cargo test --test output`; verify raw/JSON modes retain valid content and suppression does not depend on editing every command's style calls. Inspect direct writes/tracing for bypasses and document the child-output boundary.
- [x] 3.3 Reconcile README with executable command help and source: remove unsupported flags/config claims and unverified MSRV, describe target-only progress, explicit retests, relative cache behavior, safe previews, and `update --build-target`; verify examples use accepted CLI syntax and no unimplemented feature is presented as available. Preserve OpenSpec navigation and keep any openspec/README edits narrowly relevant.

## 4. Root integration and independent review

- [x] 4.1 Run final formatting checks for affected Rust files, `cargo check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`; record actual results and any preexisting blockers, resolve introduced failures, and rerun affected checks. Tests must include nonzero meaningful behavioral coverage and remain isolated from networks/user caches.
- [x] 4.2 Run `openspec validate --all --strict --no-interactive` and `git diff --check`; verify proposal, design, deltas, and implementation agree, baseline specs remain untouched until an explicit sync/archive step, and no unrelated files or `.serena/` state changed.
- [x] 4.3 Obtain independent source-and-test review against all delta scenarios and retained baseline compatibility; address blocking findings, rerun affected verification, and report residual limitations plus deferred persistence/parser/dependency hypotheses with evidence.

## Verification and review

Verified on macOS with Rust/Cargo 1.94.0:

- `cargo fmt --check`: passed.
- `cargo check --locked --offline`: passed.
- `cargo test --locked --offline`: 34 passed (3 course compatibility tests,
  2 output suites containing multiple command cases, 17 tester cases, 12 workflow cases).
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo build --locked --offline --release`: passed; the original isolated audit
  harness passed ten release regression assertions for the repaired failures.
- `openspec validate --specs --strict --no-interactive`: 7 capability specs passed;
  `openspec validate --archived`: the completed archive passed.
- `git diff --check`: passed.

The independent reviewer inspected source, new files, tests, specifications, and
verification evidence and approved without blocking findings. Additional isolated
review probes verified fresh host artifact creation and `./cmd/tester` selection,
successful-builder/missing-output rejection without a marker, missing-binary
previews without side effects or secret disclosure, and early parser color suppression.
The three build-artifact probes were subsequently added as automated regressions
and independently reviewed without blockers. The tester suite now has 17 cases;
the full suite has 34 tests.

The shell integration tests and fake tools do not establish Windows behavior or
real Go/Docker runtime compatibility. Persistence concurrency/interruption,
synthetic nested-language templates, and unmeasured dependency cleanup remain
separate follow-ups for the evidence and scope reasons recorded in design.md.
The completed delta specifications were synchronized into the main specifications,
including the new terminal-output capability, and the change was archived on
2026-11-15. Setup, planning, implementation, documentation, and the artifact-test
follow-up were recorded as separate commits without rewriting existing history.
