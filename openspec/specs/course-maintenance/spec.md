# Course Maintenance

## Purpose

Populate and maintain shared course repositories and tester binaries, and provide
local discovery and environment diagnostics for challenge workspaces.

## Requirements

### Requirement: Acquire and reuse course repositories
Initialization SHALL acquire the definition from the CodeCrafters
`build-your-own-<course>` repository and the tester from `<course>-tester`, allow
`--tester-repo` to override the tester repository name, and reuse existing cache
directories instead of cloning them again.

#### Scenario: Download an uncached course
- **WHEN** initialization finds no cached definition or tester directory
- **THEN** it uses depth-one Git clones to obtain the repositories

#### Scenario: Reuse cached repositories
- **WHEN** the requested definition and tester directories already exist
- **THEN** initialization skips cloning those directories

### Requirement: Build or reuse a platform-appropriate tester
Initialization SHALL build the tester unless `--no-build` is set. Unless a forced rebuild is requested, the builder
SHALL reuse an existing binary with an OS marker matching the resolved target. Automatic selection SHALL use local Go when available or Docker as a fallback, then verify the binary and save the actual target OS marker.

#### Scenario: Choose the tester package
- **WHEN** the tester source contains a `cmd/tester` directory
- **THEN** the build targets `./cmd/tester`; otherwise it targets `.`

#### Scenario: Build without local Go
- **WHEN** Go is unavailable and a tester build is needed
- **THEN** Docker builds a Linux tester with `CGO_ENABLED=0` using
  `golang:1.24-bookworm`
- **AND** the binary's OS marker is `linux`

#### Scenario: Skip a compatible cached build
- **WHEN** the binary exists, its marker matches the resolved build OS, and rebuilding is not forced
- **THEN** the builder returns without recompiling

#### Scenario: Initialize without building
- **WHEN** initialization includes `--no-build`
- **THEN** repository acquisition and workspace setup proceed without invoking
  the tester builder

### Requirement: Update cached repositories
The `update` command SHALL run fast-forward-only Git pulls in the workspace's cached definition and tester repositories, then force a tester rebuild. `--build-target host` SHALL require local Go; `--build-target linux` SHALL use Docker even when Go exists. With no option, selection SHALL use Go when available and Docker otherwise.

#### Scenario: Update an installed course
- **WHEN** both repository pulls succeed and a compatible cached binary exists
- **THEN** the tester is rebuilt rather than reused and successful update completion is reported only after build success

#### Scenario: A repository cannot fast-forward
- **WHEN** a repository pull fails
- **THEN** the CLI reports the Git failure and stops the update

#### Scenario: Build Linux explicitly
- **WHEN** `update --build-target linux` runs with Go available
- **THEN** Docker builds the tester and its successful build marker is `linux`

#### Scenario: Build the host explicitly
- **WHEN** `update --build-target host` runs without local Go
- **THEN** it reports that host builds require Go instead of silently building Linux

#### Scenario: Build fails
- **WHEN** the selected builder exits unsuccessfully
- **THEN** update exits unsuccessfully without announcing completion or recording a new successful OS marker

### Requirement: List known courses locally
The `list` command SHALL display its built-in course catalog without a network
lookup and mark catalog courses with directories in the selected cache as installed.

#### Scenario: Browse courses without a workspace
- **WHEN** `list` runs outside a workspace
- **THEN** the known catalog is displayed with installed markers from the cache

### Requirement: Report environment diagnostics
The `doctor` command SHALL report Git, Go, and Docker availability, the resolved
cache path, workspace detection, and installed course directories when present.
Missing optional tools or a missing workspace SHALL be reported diagnostically.

#### Scenario: Diagnose outside a workspace without Go
- **WHEN** `doctor` runs with no enclosing workspace and no Go executable
- **THEN** it explains the Docker build fallback and reports that no project is active
- **AND** it still reports the resolved cache path
