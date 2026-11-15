# CraftCoders Local CLI

Practice CodeCrafters challenges locally using cached course instructions and the official testers. The local show/test/submit workflow does not require the hosted website. Initial downloads, updates, and dependency builds can require network access.

## Install

Homebrew release automation is being prepared. Before the first verified public
release and generated tap formula are available, use the source installation
below. After publication and the tap's next successful hourly or manual update:

```sh
brew install veryshyjelly/tap/craftcoders
craftcoders --version
```

Homebrew installs a prebuilt CLI without Rust. Release archives target macOS and
Linux on ARM64 and AMD64. macOS binaries are built and smoke-tested on macOS 15;
older macOS releases are not currently validated. Linux GNU binaries are built
and tested on Ubuntu 22.04 (glibc 2.35); compatibility with older glibc or other
Linux distributions is not claimed. Hosted results establish per-platform
availability after the release workflow completes.

Build from this checkout with a Rust toolchain supporting edition 2024:

```sh
cargo install --path .
craftcoders --version
```

Git downloads course and tester repositories. Building testers requires Go, or Docker when Go is unavailable. Individual testers and your implementation may need additional runtime dependencies; consult the tester repository and run `craftcoders doctor`.

## Start a course

```sh
craftcoders list
craftcoders init redis my-redis
cd my-redis
craftcoders show
```

Initialization creates `.craftcoders.json`, `codecrafters.yml`, `compile.sh`, and `your_program.sh`. Edit the two shell scripts to build and launch your implementation. Their initial placeholders fail until configured. Any implementation language can be used through these scripts.

An existing nonempty destination is rejected. Use `init redis my-redis --no-build` to skip building the tester, or `--tester-repo NAME` to override the tester repository name.

## Instructions and progress

```sh
craftcoders status
craftcoders show 3
craftcoders show stage-slug
craftcoders show --raw
craftcoders show --json
craftcoders open
```

`show` defaults to the first unfinished enabled stage. Stage selectors accept a one-based number or slug. Raw output contains resolved Markdown; JSON includes stage metadata and resolved Markdown. `open [STAGE]` opens the stage instructions using `$EDITOR`.

Progress is stored in the workspace's `.craftcoders.json`. Workspace commands find that file by searching upward, so they can run from nested directories.

## Test and submit

```sh
craftcoders test
craftcoders test --only-current
craftcoders test --upto 3
craftcoders submit
craftcoders submit --upto 3
craftcoders test --dry-run
craftcoders submit --dry-run
```

By default, `test` runs only the current stage. `test --upto STAGE` tests the prefix through the selected target; add `--only-current` to test only that target. `submit` tests the prefix through the current stage, or through the target selected by `--upto STAGE`. Both preview commands print the planned invocation and cases without compiling, launching the tester, or saving progress.

A passing submission records only its selected target, even when the tester checks the entire prefix. Earlier unfinished stages remain unfinished. Guidance points to the first unfinished enabled stage. Failed tests and submissions preserve progress and return the tester's exit code.

After all enabled stages are complete, an untargeted test or submission reports completion. Use an explicit target such as `craftcoders test --upto 2` to retest without changing saved progress, or enable an extension.

Both test and submit accept `--docker IMAGE` to run the tester in a container with the workspace mounted at `/app`. Build a Linux tester first when needed:

```sh
craftcoders update --build-target linux
craftcoders test --docker YOUR_RUNTIME_IMAGE
```

The image must provide the tester's runtime dependencies and those of your solution. Host execution inherits the environment and runs from the workspace root. The tester receives the official `CODECRAFTERS_REPOSITORY_DIR`, `CODECRAFTERS_SUBMISSION_DIR`, and `CODECRAFTERS_TEST_CASES_JSON` variables.

## Extensions and updates

```sh
craftcoders ext list
craftcoders ext add EXTENSION_SLUG
craftcoders ext remove EXTENSION_SLUG
craftcoders update
craftcoders update --build-target host
craftcoders update --build-target linux
```

Extensions come from the course definition. Enabling one appends its stages after the base stages in saved extension order. Repeated adds are idempotent. Removing an extension retains its completed stage slugs; re-enabling it restores that progress.

Update pulls the course and tester repositories, then forces a tester rebuild. With no build target, it uses Go on the host when available and otherwise Docker for Linux. `host` requires Go; `linux` explicitly builds through Docker. The cache marker identifies the successfully built target. Updates can require network access.

## Cache and output options

The default cache is `~/.craftcoders`. Set `CC_HOME` or pass the global `--cc-home PATH` option; the flag takes precedence. Relative paths resolve against the invocation directory, including when invoked from a workspace subdirectory. Cached data lives under `courses/COURSE/{definition,tester,bin}`; the tester binary and target marker are in `bin`.

```sh
craftcoders --cc-home ./local-cache list
craftcoders --no-color status
craftcoders --verbose doctor
craftcoders --help
craftcoders update --help
```

`--no-color` or a nonempty `NO_COLOR` suppresses ANSI styling in CLI-owned stdout and stderr, including help, errors, Markdown rendering, and previews. Inherited child output from Git, builders, compilation, and testers follows those programs' own settings. `--verbose` (`-v`) enables debug logging.

Once course content, a compatible tester, and runtime dependencies are cached, instructions and local test/submission work can run offline. There is no separate offline flag or global progress/configuration file.

## Development and specifications

```sh
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

Integration tests use temporary workspaces, isolated caches, and fake external tools. Source modules are under `src/`; behavioral tests are under `tests/`.

See [the OpenSpec workflow](openspec/README.md) and [capability specifications](openspec/specs/) for the baseline and change process. [The archived workflow-hardening change](openspec/changes/archive/2026-11-15-harden-local-cli-workflows/) records the implemented contracts and verification evidence.

## Releasing

Release automation is in [`.github/workflows/release.yml`](.github/workflows/release.yml).
Branch, pull request, and manual runs validate source and packages without
publishing. The workflow pins the verified Rust 1.94.0 toolchain; this is a build
toolchain choice, not a claim about the minimum supported Rust version. Python
3.11 or newer runs the standard-library packaging helpers.

Before tagging, set the Cargo package version deliberately, run the checks,
review the changes, and commit them. Tags must exactly match `vMAJOR.MINOR.PATCH`
and Cargo metadata; prerelease tags and mismatches fail. For the current version,
the authorized release tag is `v0.1.0`:

```sh
python3 -m unittest discover -s scripts -p 'test_release.py' -v
cargo test --locked
cargo build --locked --release
python3 scripts/release.py version --tag v0.1.0
# Example on native Apple Silicon; choose the matching platform on other hosts.
python3 scripts/release.py package --tag v0.1.0 --binary target/release/craftcoders --platform darwin_arm64 --output /tmp/craftcoders-local-package
# Maintainer publication starts only when the matching stable tag is pushed.
```

The four native jobs test and build the tagged revision, check exact binary
`--version` output, and package `craftcoders_<version>_<darwin|linux>_<arm64|amd64>.tar.gz`.
Each archive contains exactly the executable `craftcoders`, `README.md`, and
`LICENSE` at its root. After all four jobs pass, the integrity job requires every
archive and writes sorted SHA-256 entries to `checksums.txt`. A complete local
four-platform collection can be checked with:

```sh
python3 scripts/release.py manifest --tag v0.1.0 --directory dist
python3 scripts/release.py validate --tag v0.1.0 --directory dist
```

Only the pushed-tag publication job has repository write permission. It creates
a draft with all four archives and the manifest, checks the uploaded asset set,
and downloads and verifies the remote bytes before making the release public.
An existing draft or published release causes a failure; reruns never overwrite
assets. A failed upload or verification leaves a draft unpublished. Inspect the
failure and draft explicitly before recovery: repair the workflow first, then
remove an incomplete unpublished draft through GitHub and rerun only after
review. Published bytes are immutable; use a new version for a published-release
correction. No cross-repository token or dispatch is required: the tap polls
public releases and verifies all four archives before generating its formula.

SHA-256 verifies consistency with the release manifest; it does not provide an
independent signature. The [upstream releases](https://github.com/veryshyjelly/craftcoders-cli/releases)
and [Homebrew tap](https://github.com/veryshyjelly/homebrew-tap) are the distribution
sources. Installation of the CLI does not install challenge tester dependencies;
Go or Docker and course-specific runtimes remain necessary when applicable.

Licensed under [MIT](LICENSE). Not affiliated with or endorsed by CodeCrafters.io.
