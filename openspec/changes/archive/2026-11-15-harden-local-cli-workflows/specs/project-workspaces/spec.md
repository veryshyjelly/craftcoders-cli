## MODIFIED Requirements

### Requirement: Initialize a challenge workspace
The CLI SHALL support `init <course> [directory]`, default the directory to
`my-<course>`, and reject a target directory that already contains entries before acquiring repositories or building testers.

#### Scenario: Initialize an empty destination
- **WHEN** a valid course is available and the destination is absent or empty
- **THEN** initialization creates `.craftcoders.json`, `codecrafters.yml`,
  `compile.sh`, and `your_program.sh` in the destination
- **AND** the saved course matches the requested slug and both extension and
  completion lists are empty
- **AND** both shell scripts are executable on Unix

#### Scenario: Reject a nonempty destination
- **WHEN** the destination contains an existing file
- **THEN** initialization reports that the destination is not empty
- **AND** no repository acquisition or tester build occurs
- **AND** it does not overwrite the destination's workspace files

#### Scenario: Reject a destination file
- **WHEN** the destination is an existing regular file
- **THEN** initialization fails before repository acquisition or tester builds and preserves that file

### Requirement: Resolve the shared cache location
The CLI MUST resolve the cache directory from `--cc-home`, then `CC_HOME`, then
`~/.craftcoders`, and store each course under `courses/<course>/` with
`definition/`, `tester/`, and `bin/` paths.

#### Scenario: Override the environment's cache
- **WHEN** `--cc-home` and `CC_HOME` both specify a directory
- **THEN** the command uses the directory supplied by `--cc-home`

#### Scenario: Use the default cache
- **WHEN** neither a CLI override nor `CC_HOME` is supplied
- **THEN** the cache is located at `.craftcoders` within the user's home directory

#### Scenario: Resolve a relative cache
- **WHEN** `--cc-home` or `CC_HOME` supplies a relative path
- **THEN** the path is resolved against the invocation working directory before any child process changes directory
- **AND** build paths and Docker mounts use absolute paths without requiring a preexisting cache
