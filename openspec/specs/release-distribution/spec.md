# release-distribution Specification

## Purpose

Distribute tested CraftCoders binaries through complete, versioned public releases that downstream installers can verify.

## Requirements

### Requirement: Validate release identity before publication
Release automation SHALL accept only stable `vMAJOR.MINOR.PATCH` tags whose numeric version equals Cargo package metadata. Untagged validation SHALL not publish releases, create tags, or push changes.

#### Scenario: Publish a matching stable tag
- **WHEN** an authorized release workflow runs for a stable tag matching the package version
- **THEN** publication remains gated on successful tests, builds, and artifact validation

#### Scenario: Reject a mismatched or unstable tag
- **WHEN** the tag is malformed, prerelease-shaped, or differs from the package version
- **THEN** publication fails without creating a public release

### Requirement: Produce complete platform archives
A stable release SHALL contain `craftcoders_<version>_<os>_<arch>.tar.gz` for each combination of `darwin` and `linux` with `arm64` and `amd64`. Every archive SHALL include the executable `craftcoders`, `README.md`, and `LICENSE` at its root. Each binary SHALL pass a matching-platform `--version` smoke test.

#### Scenario: Inspect a supported archive
- **WHEN** an archive is unpacked for a supported platform and architecture
- **THEN** its executable is directly installable without a Rust compiler
- **AND** `craftcoders --version` reports the release version

### Requirement: Publish checksums and complete assets together
Releases SHALL include `checksums.txt` with one SHA-256 entry for each archive, using a lowercase 64-digit digest, two spaces, and the exact archive basename. Publication SHALL occur only after all four archives and their manifest are validated and attached; failed builds or uploads SHALL not expose a newly incomplete public release.

#### Scenario: Complete all build jobs
- **WHEN** four platform archives pass validation
- **THEN** publication provides all four archives and their matching checksum manifest

#### Scenario: An asset fails verification or upload
- **WHEN** an archive is missing, has an invalid version, or fails upload
- **THEN** the release remains unpublished and the workflow reports failure

### Requirement: Use repository-scoped release permissions
Release builds and tests SHALL run without repository write permissions. Only the publication job SHALL have contents-write permission, scoped to the CLI repository; distributing through the tap SHALL require no cross-repository credentials.

#### Scenario: Validate an ordinary source change
- **WHEN** validation runs for a branch or pull request
- **THEN** it can test and package without permission to publish a release
