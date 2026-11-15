# Tester Execution

## Purpose

Run official challenge testers against local implementations, select test stages,
and update workspace completion only after a successful submission.

## Requirements

### Requirement: Select test cases from enabled stages
The `test` and `submit` commands SHALL default to testing every enabled stage from
the beginning through the current stage. `--upto <number-or-slug>` SHALL select
the target, and `test --only-current` SHALL restrict execution to that target.

#### Scenario: Test a prefix
- **WHEN** the target is the third enabled stage and `--only-current` is absent
- **THEN** the tester receives cases for stages one through three in order

#### Scenario: Test only an explicit target
- **WHEN** `test --only-current --upto 3` runs with at least three enabled stages
- **THEN** the tester receives only the third stage's case

#### Scenario: No current stage remains
- **WHEN** `test` or `submit` runs after every enabled stage is complete
- **THEN** it reports that all stages are complete and suggests enabling extensions

### Requirement: Compile before executing tests
Non-preview test and submit execution SHALL run `compile.sh` from the workspace
root when it exists and stop with an error if compilation fails.

#### Scenario: Compilation fails
- **WHEN** `compile.sh` exits unsuccessfully before a test run
- **THEN** the CLI reports compilation failure
- **AND** the tester does not run and completion is not saved

### Requirement: Honor the official tester environment contract
Tester execution SHALL supply `CODECRAFTERS_REPOSITORY_DIR`,
`CODECRAFTERS_SUBMISSION_DIR`, and `CODECRAFTERS_TEST_CASES_JSON`. Each case SHALL
include a slug, `stage-<number>` log prefix, and `Stage #<number>: <name>` title.

#### Scenario: Execute on the host
- **WHEN** a tester runs without `--docker`
- **THEN** repository and submission variables reference the absolute workspace path
- **AND** the tester inherits stdin, stdout, and stderr for interactive streaming

### Requirement: Support Docker tester execution
With `--docker <image>`, the CLI SHALL mount the workspace at `/app`, mount the
tester read-only at `/usr/local/bin/tester`, use `/app` as the working directory
and both tester directory variables, and reject a known non-Linux tester marker.

#### Scenario: Execute in a container
- **WHEN** a Linux tester is available and `--docker` specifies an image
- **THEN** the CLI runs that image with the mounted tester and selected test cases

#### Scenario: Reject an incompatible marked binary
- **WHEN** Docker mode reads a tester OS marker other than `linux`
- **THEN** it reports that Docker requires a Linux tester instead of launching it

### Requirement: Preview the tester invocation
With `--dry-run`, the CLI SHALL print the tester command and formatted test cases
without launching the tester or persisting completion. The `test` command SHALL
also skip compilation in this mode.

#### Scenario: Preview a test run
- **WHEN** `test --dry-run` selects valid stages with a compatible tester marker
- **THEN** it prints the planned command and cases
- **AND** neither compilation nor the tester executes

#### Scenario: Preview a submission
- **WHEN** `submit --dry-run` reaches the tester preview after its compilation step
- **THEN** the tester does not execute and completion remains unchanged

### Requirement: Preserve test exit status without saving progress
The `test` command SHALL exit with the tester's exit code and leave the saved
completion list unchanged, regardless of whether the tester passes.

#### Scenario: A test fails
- **WHEN** the tester exits with code `1` during `test`
- **THEN** the CLI exits with code `1` and does not mark the target complete

### Requirement: Save the submission target only after success
A non-preview `submit` SHALL append the selected target slug to `completed` only
when the tester returns zero, save the workspace, and announce the next enabled
stage or the end of the enabled course.

#### Scenario: Submit the current stage successfully
- **WHEN** the tester passes for a normal submission
- **THEN** the target slug is saved and completion is announced

#### Scenario: Submit a later target successfully
- **WHEN** `submit --upto 3` passes while earlier stages remain unfinished
- **THEN** only the third stage's slug is appended to completed
- **AND** earlier unfinished stages remain eligible to be current

#### Scenario: Submission fails
- **WHEN** the tester returns a nonzero exit code during submission
- **THEN** the CLI exits with that code and does not save completion
