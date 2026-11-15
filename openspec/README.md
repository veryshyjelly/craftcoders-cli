# OpenSpec workflow

This directory records the current behavior of CraftCoders and plans for future
changes. It was initialized with OpenSpec 1.16.1 using the `spec-driven` schema.

## Layout

- `config.yaml`: project constraints and artifact rules.
- `specs/<capability>/spec.md`: the implemented behavior baseline.
- `changes/<change-name>/`: a proposed change's proposal, design, delta specs,
  and tasks.
- `changes/archive/`: completed changes.
- `../.agents/skills/openspec-*/`: generated Codex workflow skills.

## Baseline capabilities

| Capability | Implementation |
| --- | --- |
| [Project workspaces](specs/project-workspaces/spec.md) | `src/config.rs`, `src/project.rs`, `src/commands.rs` |
| [Course stages](specs/course-stages/spec.md) | `src/course.rs`, `src/commands.rs` |
| [Stage instructions](specs/stage-instructions/spec.md) | `src/course.rs`, `src/render.rs`, `src/commands.rs` |
| [Tester execution](specs/tester-execution/spec.md) | `src/tester.rs`, `src/commands.rs`, `src/main.rs` |
| [Course extensions](specs/course-extensions/spec.md) | `src/course.rs`, `src/commands.rs` |
| [Course maintenance](specs/course-maintenance/spec.md) | `src/git_ops.rs`, `src/tester.rs`, `src/commands.rs` |

The specs were derived from source, rather than from the root README's feature
list. There is no implemented global `config.toml`, global progress file,
extension repository management, `cleanup`, `completion`, `--offline`,
`status --current-only`, or JUnit output. These need proposals before being
treated as supported behavior.

Current implementation caveats to consider when planning changes:

- `submit --dry-run` still runs `compile.sh` if present, although it skips the
  tester and does not save completion.
- `submit --upto` saves only the selected target slug, even though it tests the
  whole prefix. Earlier unfinished stages remain unfinished.
- `update` pulls both repositories and calls the tester builder, but the builder
  can reuse an existing binary when its OS marker matches. It does not guarantee
  a rebuild after a source update.

## Create and implement a change

In Codex, invoke the installed skills by name:

```text
$openspec-propose "describe the change"
$openspec-apply-change
$openspec-archive-change
```

Use `openspec-explore` to investigate an idea, `openspec-update-change` to revise
planning artifacts, and `openspec-sync-specs` to sync deltas before archiving.
Restart the Codex session if newly installed skills are not visible.

For a manual CLI workflow, replace `change-name` with a descriptive kebab-case
name:

```sh
openspec new change change-name
openspec status --change change-name
openspec instructions proposal --change change-name
```

Write the proposal, then use `openspec instructions <artifact> --change
change-name` for `specs`, `design`, and `tasks` as they become ready. Record
additions, modifications, and removals in delta specs under the change directory.
Keep the main specs as the implemented baseline until the change is completed
and synced or archived.

Validate artifacts and perform verification appropriate to the implementation
before marking tasks complete:

```sh
openspec validate --all --strict --no-interactive
```

For Rust changes, relevant checks include `cargo fmt --check`, `cargo check`, and
`cargo test`. The repository currently has no automated test cases; add focused
coverage when introducing behavior that needs it. External-process checks should
use fixtures or fake executables and an isolated `CC_HOME` where practical.

After implementation and verification, archive the change and sync its deltas:

```sh
openspec archive change-name
```

Inspect the resulting main-spec updates before committing. Browse the baseline
with `openspec list --specs`; refresh generated skills with `openspec update`.
