## Purpose

Install CraftCoders through the owner's Homebrew tap using platform-specific binaries verified from stable upstream releases.

## ADDED Requirements

### Requirement: Consume only complete published stable releases
The tap updater SHALL consume public, non-draft, non-prerelease releases with stable `vMAJOR.MINOR.PATCH` tags. It SHALL require all four named platform archives and `checksums.txt` before generating a CraftCoders formula. Before the first release, a release-not-found response SHALL leave the tap unchanged and succeed.

#### Scenario: No release exists yet
- **WHEN** the upstream latest-release endpoint returns HTTP 404
- **THEN** the updater reports that it is skipping and creates no formula

#### Scenario: An upstream request fails otherwise
- **WHEN** the endpoint returns an authentication, rate-limit, server, or other non-404 failure
- **THEN** the updater reports failure rather than treating it as an absent release

#### Scenario: A release is ineligible
- **WHEN** the release is draft, prerelease, malformed, or missing a required asset
- **THEN** the updater fails without modifying the existing formula

### Requirement: Verify all downloads before atomic replacement
The updater SHALL validate checksum syntax and unique safe basenames, download all required archives, and compare each SHA-256 digest with the manifest before atomically replacing `Formula/craftcoders.rb`. It SHALL never execute downloaded release content during verification.

#### Scenario: Every checksum matches
- **WHEN** all four downloaded archives match the valid manifest
- **THEN** the formula is replaced with URLs and digests from that verified release

#### Scenario: Verification or replacement preparation fails
- **WHEN** a checksum is malformed or duplicated, an archive is corrupt, or a download/write fails before replacement
- **THEN** the existing formula remains byte-for-byte unchanged and no partial formula appears

#### Scenario: Generate and repair readable formula permissions
- **WHEN** verified release content is generated or an identical formula has restrictive permissions
- **THEN** the formula is atomically written with mode `0644`
- **AND** preparation failures preserve the previous formula bytes and clean up temporary output

### Requirement: Install the native CLI binary
The generated formula SHALL select the correct macOS/Linux ARM64/AMD64 archive and verified digest, install `craftcoders` into Homebrew's binary directory, declare its MIT license, and test `--version`. It SHALL not require Rust to install the prebuilt CLI.

#### Scenario: Install on a supported platform
- **WHEN** a user installs `veryshyjelly/tap/craftcoders` after the first verified formula is available
- **THEN** Homebrew selects the appropriate archive and the formula test checks the installed version

### Requirement: Integrate with the existing tap polling workflow
The existing hourly and manual tap workflow SHALL run the CraftCoders updater and its tests, check generated formula syntax when present, and include that formula in its existing update commit. The addition SHALL preserve existing Eitri and Vocabulary Studio package behavior.

#### Scenario: Run the tap workflow before the first release
- **WHEN** no CraftCoders release exists and other package updates succeed
- **THEN** the workflow continues without requiring a CraftCoders formula or new credentials
