# Design

## Context

The CLI package is `craftcoders` version `0.1.0`, edition 2024, with MIT declared in Cargo and README but no LICENSE file. Cargo's repository URL currently points to the unrelated organization root. There are no tags, releases, or release workflows. CLI behavior already has regression tests and a side-effect-free `--version` command.

The clean sibling repository `../homebrew-tap` uses its own contents-write token in `.github/workflows/update.yml`, running hourly and manually. `scripts/update.py` handles Eitri; `scripts/update_vocabulary_studio.py` supplies a useful stable-tag, first-release-404, download-verification pattern. Its current tests mock downloads with fixtures. Existing updater writes are not atomic; this change adds atomic replacement only for the new CraftCoders updater rather than altering other packages. No AGENTS.md was found in either repository or their ancestor directories; the conversation's repository routing instructions apply.

## Goals / Non-Goals

**Goals:** Establish a shared immutable asset naming contract so CLI and tap owners can work independently; validate locally without real publishing; preserve existing tap packages and local runtime behavior.

**Non-Goals:** No crates.io publishing, package version bump, automatic tag creation, cross-repository dispatch/token, code-signing/notarization pipeline, source-build formula fallback, unsupported platform claims, or unrelated tap refactoring. The owner explicitly authorized publishing version 0.1.0 after verification; root performs the release actions after the gates pass.

## Decisions

### Native build matrix and shared artifact contract

Use native GitHub-hosted runners and standard GNU Linux Rust targets to avoid cross-compilation tooling:

| Archive suffix | Rust target | Runner |
| --- | --- | --- |
| darwin_arm64 | aarch64-apple-darwin | macos-15 |
| darwin_amd64 | x86_64-apple-darwin | macos-15-intel |
| linux_arm64 | aarch64-unknown-linux-gnu | ubuntu-22.04-arm |
| linux_amd64 | x86_64-unknown-linux-gnu | ubuntu-22.04 |

Runner labels/architectures were checked against current official GitHub documentation. Document Ubuntu 22.04/glibc 2.35 as the Linux build baseline, and the macOS versions actually validated; do not claim testing on older macOS or arbitrary Linux distributions. Native stable Rust builds use `--locked`; release jobs test the exact tagged revision and run the built binary's `--version` on its matching runner. Pin an explicit toolchain only if a verified project-supported version is chosen; do not invent an MSRV.

Names are `craftcoders_<version>_<darwin|linux>_<arm64|amd64>.tar.gz`, where version has no `v`. Each archive has exactly the root entries `craftcoders`, `README.md`, and `LICENSE`; binary permissions must remain executable. `checksums.txt` contains exactly four sorted entries of lowercase SHA-256 hex, two spaces, and basename, newline-terminated. Do not include workflow artifact ZIPs or the manifest itself. The tap validates the exact required archive set; harmless unrelated upstream release assets may exist but are never downloaded or executed.

Alternative: cross-compile static musl or use a release orchestration framework. Rejected for this first integration because native builds permit real per-platform smoke tests and add no new toolchain framework. Alternative: source-build Homebrew formula. Rejected because it introduces Rust build prerequisites and bypasses the requested verified-binary release contract.

### Release gates and permission separation

Create one CLI workflow with read-only validation/build jobs and a separate contents-write publication job. Run normal branch/PR validation without publishing; allow a manual validation run if useful. Only an existing pushed stable tag can enter publication. Require regex `v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)`, exact equality with Cargo metadata, and matching binary output `craftcoders <version>`. Pass event/tag values via environment/arguments, never interpolate untrusted text into shell source. Check out the triggering revision and disable credential persistence where write access is unnecessary. Serialize releases per tag without cancelling active publication.

After all native jobs pass, download their artifacts, require all four archives, validate archive contents/modes and checksum manifest, then use the CLI repository token to create a draft with all five assets. Verify the uploaded asset set before publishing the draft. `gh release create --verify-tag --draft` and `gh release edit --draft=false` are documented by the official GitHub CLI manual. Do not overwrite an already published version or silently replace immutable assets. A rerun encountering an existing published release fails with guidance; a preexisting draft is left for explicit recovery rather than guessed cleanup. A failed build or upload leaves no new public partial release. Setup itself invokes no publishing commands.

Alternative: publish first and upload later. Rejected because the hourly tap could observe a partial public release. Alternative: push a formula from CLI CI. Rejected because the existing tap poller already provides separation without cross-repository secrets.

### Dedicated verified tap updater

Add `scripts/update_craftcoders.py` with independently testable `archive_names(tag)`, `parse_checksums(text)`, `render(tag, checksums)`, `latest_release()`, `download(tag, name, directory)`, and `update(target=None)` functions, following the Vocabulary Studio script's boundary pattern. Default repository is `veryshyjelly/craftcoders-cli` and target is `Formula/craftcoders.rb`. Only exact HTTP 404 from latest-release lookup skips; other API failures propagate. Reject draft/prerelease/unsafe tags before download, require the manifest and all archives in release assets, and reject ambiguous duplicate required asset names.

Accept standard SHA-256 manifest syntax with optional binary `*` marker and blank lines, normalize hex case, and reject malformed digests, duplicate entries, missing names, path separators/traversal, or unexpected archive names. Require the four expected filenames; verify all downloaded bytes by streaming hashes before rendering/writing. URLs are constructed from the trusted repository constant, validated tag, and known basenames, not arbitrary API asset URLs. Downloading does not authorize executing contents. Create a temporary formula in the destination directory, flush/close it, and replace via `os.replace`; clean up temporary files on failure. This guarantees no partial replacement, not cross-process locking or crash-proof filesystem durability. Avoid rewriting identical formula content where practical.

The formula class is `Craftcoders`, with MIT license and GitHub homepage, `on_macos`/`on_linux` plus ARM/Intel branches, verified versioned archive URLs and digests, `bin.install "craftcoders"`, and an exact version smoke test. No Go/Rust build dependencies: Go or Docker requirements for later challenge tester builds belong in user documentation. There is no formula checked in before genuine public artifacts exist. Fixture formulas live only in temporary test output.

Extend the existing tap workflow by running the new updater, syntax-checking the formula only when present, and conditionally staging it in the existing bot commit. Keep the current hourly schedule, permissions, concurrency, Eitri/Vocabulary Studio calls, and error propagation intact.

### File ownership and sandbox-safe tap integration

**CLI release implementer:** `.github/workflows/release.yml`, `scripts/release.py`, `scripts/test_release.py`, `Cargo.toml` repository metadata only, new `LICENSE`, and CLI `README.md`. The helper exposes packaging/validation as locally testable operations; its internal CLI can be chosen by the implementer, but filenames/contents and version gates above are fixed. No src/ changes are needed. Use MIT text with copyright year 2026 and existing author Ayush Biswas; no additional ownership claim is inferred.

**Tap implementer:** Work in a temporary copy of the sibling tap, editing only `scripts/update_craftcoders.py`, `scripts/test_update_craftcoders.py`, `.github/workflows/update.yml`, and tap `README.md`. Deliver an exact patch and passing tests to root. Do not edit existing updater/test scripts, real formulae, or the actual sibling directory. Root reviews the patch, requests the required sandbox escalation for the concrete sibling writes, applies it, and reruns tests in the real tap. The generated path `Formula/craftcoders.rb` is not part of the initial patch; it is added only after the authorized real release is available and verified.

**Root:** Owns planning task status, integration verification, approved patch application, and meaningful verified commits in each repository. The owner authorized required pushes, tagging, and publication of v0.1.0 after verification. Root performs those steps after local gates, waits for all hosted build jobs, and verifies the generated formula/install. Do not infer a version bump.

### Verification and documentation

CLI tests use fake executables/archive fixtures to reject tag/version mismatch, missing binary, wrong version output, wrong archive contents/modes, missing matrix outputs, and malformed manifests; a local real release build/package smoke complements fixtures. Workflow validation must inspect dependencies/conditions and permissions; use actionlint where available. Per-platform runtime proof comes from hosted CI, and must not be claimed before those jobs run.

Tap tests cover all four platform mappings, exact URLs/version output, invalid tags/digests/paths, duplicate entries/assets, missing assets, corrupt downloads, HTTP 404 versus other errors, atomic replacement and preservation on failures, and unchanged existing package tests. Run Python unittest discovery and Ruby syntax checks against a temporary generated fixture formula. Real brew audit/install follows genuine release assets/formula creation in the authorized release sequence; do not substitute fabricated checksums.

Both READMEs distinguish preparation from availability: explain the future `brew install veryshyjelly/tap/craftcoders` path, current lack of a first release when applicable, platform baseline, local CLI runtime versus tester build prerequisites, release steps, tap polling, and recovery for a failed draft. Preserve existing package docs.

## Risks / Trade-offs

- Four native runners cost more than a single cross-build → they provide direct smoke evidence and avoid cross toolchains.
- GNU builds depend on the supported Linux baseline → document it and validate native binaries; static-musl distribution is a separate enhancement.
- SHA-256 verifies artifact consistency, not upstream identity independent of GitHub → both source and manifest come from the trusted public repository; no signature claim is made.
- Hosted runners/action versions can change → verify official support during implementation and let CI failures block publication.
- Tap 404 can also indicate an unavailable/private repository → the upstream is expected public; other HTTP errors fail visibly and docs state the prerequisite.
- No first release means no immediately installable formula → this is deliberate; the updater creates it after verification instead of inventing release data.

## Migration Plan

No runtime state migration. Land verified setup commits in CLI and tap independently; neither requires a public release to pass local tests. After an explicitly authorized stable tag release completes, the tap's next hourly or manually triggered update verifies artifacts and commits the formula. Rollback of setup removes the new workflow/updater wiring without changing existing packages or user workspaces. Do not mutate published artifact bytes to repair a release; use a new version when needed.

## Open Questions

None. The owner authorized publishing `v0.1.0` after verification; sandbox approval for applying the concrete tested sibling patch is an execution permission, not an unresolved product decision.
