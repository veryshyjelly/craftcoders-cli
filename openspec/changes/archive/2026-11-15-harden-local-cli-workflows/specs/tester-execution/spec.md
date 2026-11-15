## MODIFIED Requirements

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
- **WHEN** `test` without an explicit target or `submit` runs after every enabled stage is complete
- **THEN** it reports that all stages are complete and suggests enabling extensions

#### Scenario: Retest after completing the course
- **WHEN** all stages are complete and `test --upto 2` selects an enabled stage
- **THEN** the tester runs the requested stage prefix without changing completion

#### Scenario: Reject an invalid target before compilation
- **WHEN** a test or submission selector matches no enabled stage
- **THEN** the command fails before compilation or tester execution and leaves completion unchanged

### Requirement: Honor the official tester environment contract
Tester execution SHALL supply `CODECRAFTERS_REPOSITORY_DIR`,
`CODECRAFTERS_SUBMISSION_DIR`, and `CODECRAFTERS_TEST_CASES_JSON`. Each case SHALL
include a slug, `stage-<number>` log prefix, and `Stage #<number>: <name>` title.

#### Scenario: Execute on the host
- **WHEN** a tester runs without `--docker`
- **THEN** repository and submission variables reference the absolute workspace path
- **AND** its working directory is the absolute workspace root
- **AND** the tester inherits the ambient environment with only the official contract variables overridden
- **AND** the tester inherits stdin, stdout, and stderr for interactive streaming

### Requirement: Preview the tester invocation
With `--dry-run`, both `test` and `submit` SHALL print the planned tester command and formatted cases without compiling, launching the tester, or persisting completion. Preview output SHALL exclude unrelated inherited environment values and handle Unicode paths and values without panicking.

#### Scenario: Preview a test run
- **WHEN** `test --dry-run` selects valid stages with a compatible tester marker
- **THEN** it prints the planned command and cases
- **AND** neither compilation nor the tester executes

#### Scenario: Preview a submission
- **WHEN** `submit --dry-run` selects valid stages with a compatible tester marker
- **THEN** neither compilation nor the tester executes and completion remains byte-for-byte unchanged

#### Scenario: Preview with a secret and Unicode
- **WHEN** the parent environment includes an unrelated synthetic secret and the workspace path contains multibyte characters
- **THEN** the preview succeeds without exposing the secret or splitting a UTF-8 character


### Requirement: Save the submission target only after success
A non-preview `submit` SHALL append the selected target slug to `completed` only
when the tester returns zero, save the workspace, and announce the first unfinished enabled
stage from saved completion, or the end only when none remains.

#### Scenario: Submit the current stage successfully
- **WHEN** the tester passes for a normal submission
- **THEN** the target slug is saved and completion is announced

#### Scenario: Submit a later target successfully
- **WHEN** `submit --upto 3` passes while earlier stages remain unfinished
- **THEN** only the third stage's slug is appended to completed
- **AND** earlier unfinished stages remain eligible to be current
- **AND** guidance names the first unfinished stage rather than declaring the enabled course finished

#### Scenario: Submission fails
- **WHEN** the tester returns a nonzero exit code during submission
- **THEN** the CLI exits with that code and does not save completion
