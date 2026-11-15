# Proposal

## Why

The local audit reproduced secret disclosure and a Unicode panic in previews, inconsistent submission side effects, broken nested workspace discovery, stale tester builds, and misleading output/documentation. The repository currently has no automated tests to protect these workflows.

## What Changes

- Make test and submit previews safe, Unicode-safe, and free of compilation, execution, and progress writes.
- Resolve nested workspaces and relative cache paths consistently, and run host testers from the workspace root.
- Preserve target-only submission completion while reporting the actual first unfinished stage; permit explicit test targets after completion.
- Force tester rebuilds after updates and add `update --build-target host|linux`, retaining automatic selection when omitted.
- Reject invalid initialization destinations before repository or build work.
- Honor global color suppression across CLI-owned output and correct README claims to match the implemented CLI.
- Add isolated behavioral tests with fake tools, without network downloads or access to user caches.

## Capabilities

### New Capabilities

- `terminal-output`: Consistent color suppression for CLI-owned output.

### Modified Capabilities

- `tester-execution`: Safe previews, explicit retests, root execution, and accurate submission guidance.
- `course-maintenance`: Forced update rebuilds and explicit build-target selection.
- `project-workspaces`: Early destination validation and invocation-relative absolute cache resolution.

## Impact

Changes affect commands, argument parsing, configuration/path resolution, tester process creation, terminal rendering, README, and new tests. Existing workspace JSON, stage order, extension retention, official tester environment, and cache layout remain compatible. Updates and initial builds may still use networks; cached local workflows and regression tests must not require them. No parser rewrite, dependency cleanup, persistence/locking redesign, or implementation of aspirational README features is included.
