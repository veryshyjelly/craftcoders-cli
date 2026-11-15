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

## Capabilities

### New Capabilities

- `release-distribution`: Versioned, complete, verified CLI archives and their release automation.
- `homebrew-distribution`: Verified stable-release ingestion and platform-specific formula generation in the owner's tap.

### Modified Capabilities

None. Existing local command behavior, workspace formats, course caches, and tester contracts remain unchanged.

## Impact

CLI changes are limited to release/CI scripts, workflow definitions, metadata/license, tests, and documentation. Tap changes are limited to a new updater/tests, the existing update workflow, documentation, and a generated formula only after a real release. The tap keeps its own repository token; no cross-repository PAT, website dependency, or existing-package behavior change is required. The owner has authorized publishing v0.1.0 after verification, including necessary CLI/tap pushes and tagging; release execution remains gated on passing checks.
