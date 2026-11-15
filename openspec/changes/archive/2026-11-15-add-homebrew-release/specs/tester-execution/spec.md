## MODIFIED Requirements

### Requirement: Select test cases from enabled stages
The `test` command SHALL default to testing only the current enabled stage.
The `submit` command SHALL default to testing every enabled stage from the
beginning through the current stage. `--upto <number-or-slug>` SHALL select the
target and test its prefix; `test --only-current` SHALL restrict execution to
that target.

#### Scenario: Test a prefix
- **WHEN** `submit` targets the third enabled stage or `test --upto 3` runs without `--only-current`
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

#### Scenario: Test the current stage by default
- **WHEN** the third enabled stage is current and `test` runs without `--upto`
- **THEN** the tester receives only the third stage
- **AND** saved completion remains unchanged
