# Tasks

Groups 1 and 2 can proceed independently using design.md's exact four-archive/checksum contract. Root updates checkboxes after evidence and owns external release execution. The owner authorized publishing v0.1.0 after verification.

## 1. CLI release implementer

- [x] 1.1 Correct Cargo repository metadata and add MIT LICENSE using existing author/year evidence; verify `cargo metadata --no-deps` reports the correct repository and package version remains 0.1.0, with no runtime changes.
- [x] 1.2 Add `scripts/release.py` packaging/version/manifest validation plus `scripts/test_release.py`; test stable tag matching, executable smoke-version agreement, exact archive names/root entries/modes, four-output completeness, and invalid/missing input rejection using temporary fixtures.
- [x] 1.3 Add `.github/workflows/release.yml` with the four native runner/target pairs, locked tests/builds and native version smoke checks, read-only build permissions, and tag-only gated publication; validate YAML/actions syntax and inspect job dependencies so failed tests/builds cannot publish.
- [x] 1.4 Implement complete draft asset upload followed by verification/publication with the CLI repository token, existing-tag verification, and refusal to overwrite existing releases; verify command construction/gating locally without network publication and document failed-draft recovery.
- [x] 1.5 Document release process, supported build baselines, Homebrew availability/prerequisites, and exact install command in CLI README; verify examples against packaging interfaces and preserve existing local-workflow guidance. Run local packaging unit tests, `cargo test --locked`, and a native release-package `--version` smoke test before handing off.

## 2. Tap implementer — temporary copy only

- [x] 2.1 Add `scripts/update_craftcoders.py` with stable release lookup, precise 404 skip, safe manifest parsing, complete asset verification, platform formula rendering, and atomic replacement; verify expected rendered URLs/digests/license/install/version test for all four platforms.
- [x] 2.2 Add `scripts/test_update_craftcoders.py` covering malformed/prerelease tags, drafts, missing/duplicate assets, invalid/duplicate/path-unsafe checksums, corrupt downloads, 404 versus other HTTP failures, and preservation of an existing formula on every pre-replacement failure; run all existing and new tap Python tests without real downloads.
- [x] 2.3 Extend only the existing tap workflow and README for CraftCoders polling/testing/conditional syntax-check/staging; verify existing package calls, cadence, permissions, and docs remain intact, and absent first-release formula is handled cleanly.
- [x] 2.4 Generate a temporary fixture formula and validate Ruby syntax plus available Homebrew style checks; deliver the exact restricted-file patch and test evidence to root without writing to the sibling repository or committing fabricated release checksums.

## 3. Root integration and pre-publication verification

- [x] 3.0 Merge upstream commit `7a721fa` while preserving both histories and the owner-approved current-stage default for `test`; retain submit/explicit-upto prefix execution and local hardening guards, resolve source/help/README/test expectations coherently, and verify all five incoming stage-selection tests plus affected workflow regressions before re-running task 3.2.

- [x] 3.1 Review the tap patch, obtain required sandbox permission to apply its concrete sibling writes, apply it, and rerun the full tap test suite and workflow checks in the real repository; verify only the agreed files changed and no existing package formula changed.
- [x] 3.2 Run CLI packaging tests, `cargo check --locked`, `cargo test --locked`, appropriate formatting/lint checks, `git diff --check`, and `openspec validate --all --strict --no-interactive`; obtain independent review, fix blocking findings, rerun affected checks, and commit verified setup milestones separately in CLI and tap.

## 4. Root authorized v0.1.0 release and Homebrew verification

- [x] 4.1 Push verified CLI and tap setup commits, create/push the matching v0.1.0 tag, and observe the release workflow; verify all four native test/build/version jobs and aggregate integrity checks pass before publication. On failure, fix the cause and recover explicitly without overwriting an existing published release.
- [x] 4.2 Verify the published v0.1.0 release is stable and contains the four exact archives plus checksums.txt; download and hash all archives, inspect required archive files/executable modes, and confirm recorded runner smoke evidence for all four platforms.
- [x] 4.3 Run or trigger the tap's verified updater after release availability, verify the real Formula/craftcoders.rb contains only actual release digests/URLs, run tap tests/Ruby/Homebrew checks, and commit/push the generated formula through the established updater workflow or an equivalent verified local update.
- [x] 4.4 Install/test the real formula on the available native Homebrew platform and verify `craftcoders --version` reports 0.1.0; report tested platform and any remaining remote-platform installation limitations accurately. Confirm public release/tap links and repository status before declaring completion.

## Release and verification evidence

- Published stable release: https://github.com/veryshyjelly/craftcoders-cli/releases/tag/v0.1.0
- Annotated tag `v0.1.0` points to `4127a1a03343190cbbd0eb21c4694490428ce0e8`.
- Untagged native validation: https://github.com/veryshyjelly/craftcoders-cli/actions/runs/43716484618
- Tagged native builds, integrity gate, and publication: https://github.com/veryshyjelly/craftcoders-cli/actions/runs/43718343799
- Verified tap generation: https://github.com/veryshyjelly/homebrew-tap/actions/runs/43722948580
- Final tap workflow, including audit fixes: https://github.com/veryshyjelly/homebrew-tap/actions/runs/43733584168

All four native jobs passed their tests, release build, and version/package smoke
checks. The integrity job verified the complete four-archive set and manifest
before the draft was published. Root separately downloaded the five public
assets and validated SHA-256 digests, exact archive root entries, and executable
modes. The actual formula's four URLs and digests match that manifest.

Local verification passed: 39 Rust tests, 21 release-helper tests, 33 tap tests,
Rust formatting/check/strict Clippy, both Actions workflows through Actionlint,
Ruby syntax, real formula style, and `brew audit --strict --online`. Homebrew
installed the ARM64 macOS release and `brew test` passed; the installed Mach-O
ARM64 binary reports `craftcoders 0.1.0`. Homebrew installation was tested only
on this native Mac; Linux and Intel binaries have native hosted smoke evidence,
not a claim of local Homebrew installation on those systems.

Independent review caught and resolved draft lookup through the published-tag
REST endpoint before any release tag was pushed. Publication now detects drafts
through authenticated pagination, resolves new drafts through the draft-aware
CLI lookup, and verifies remote bytes before publishing. Real Homebrew checks
also drove the version-inference and readable atomic-file fixes; regressions
cover new/replaced/identical formula permissions and chmod failure preservation.
No published release artifact was overwritten to make those tap metadata fixes.

Both source repository origins use SSH. Existing history was preserved, including
upstream `7a721fa` and its five tests, through merge `79f53b8`; the owner explicitly
selected its current-stage default. Setup, planning, implementation, tests,
documentation, release automation, and tap fixes were committed separately.
Local `.serena` metadata was intentionally excluded from commits.
