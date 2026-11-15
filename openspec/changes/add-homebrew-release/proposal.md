# Proposal

## Why

CraftCoders has no published release artifacts or Homebrew installation path, while the owner's existing tap already polls and verifies public releases for other tools. Add a release contract and a compatible tap updater so users can install a verified prebuilt CLI without Rust.

## What Changes

- Build and smoke-test versioned CLI archives for macOS and Linux on ARM64 and AMD64, with SHA-256 checksums, in GitHub Actions.
- Validate stable tags against Cargo metadata and publish complete release assets only after checks pass, using repository-scoped permissions.
- Add a dedicated CraftCoders updater to the existing sibling tap's hourly workflow; verify all four downloads before atomically generating a formula.
- Skip cleanly before the first public release; do not create a placeholder formula or invented checksums.
- Correct package repository metadata, include the already-declared MIT license text, and document releases and installation availability.
- Test packaging and updater failure cases with local fixtures before applying tap changes.
- Integrate upstream commit `7a721fa`, preserving its owner-approved current-stage default for `test` while retaining prefix execution for `submit` and explicit `test --upto`.

## Capabilities

### New Capabilities

- `release-distribution`: Versioned, complete, verified CLI archives and their release automation.
- `homebrew-distribution`: Verified stable-release ingestion and platform-specific formula generation in the owner's tap.

### Modified Capabilities

- `tester-execution`: Preserve the owner-approved upstream current-stage default for `test`; submission and explicit prefix selection remain supported. Workspace formats, course caches, and the official tester environment remain unchanged.

## Impact

CLI changes cover release/CI scripts, workflow definitions, metadata/license, tests, documentation, and the necessary owner-approved integration of upstream stage-selection behavior in CLI argument/help and command orchestration. Tap changes are limited to a new updater/tests, the existing update workflow, documentation, and a generated formula only after a real release. The tap keeps its own repository token; no cross-repository PAT, website dependency, or existing-package behavior change is required. The owner has authorized publishing v0.1.0 after verification, including necessary CLI/tap pushes and tagging; release execution remains gated on passing checks.
