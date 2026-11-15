# Project Workspaces

## Purpose

Create local challenge workspaces and persist course selection, enabled
extensions, and completed stage slugs independently of the shared course cache.

## Requirements

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

### Requirement: Discover and load project state
Project commands SHALL locate the nearest `.craftcoders.json` by walking upward
from the working directory and load its `course`, `extensions`, and `completed`
fields, defaulting absent extension and completion lists to empty lists.

#### Scenario: Run from a workspace subdirectory
- **WHEN** a project command runs in a nested directory of an initialized workspace
- **THEN** it uses the enclosing workspace's configuration

#### Scenario: Run outside a workspace
- **WHEN** a project command cannot find `.craftcoders.json` in any ancestor
- **THEN** it reports that the directory is not inside a CraftCoders project and
  directs the user to initialize one

#### Scenario: Load a minimal existing configuration
- **WHEN** `.craftcoders.json` contains a course but omits extensions and completed
- **THEN** the configuration loads with empty extension and completion lists

### Requirement: Persist workspace progress locally
Workspace mutations SHALL save enabled extension slugs and completed stage slugs
in the workspace's `.craftcoders.json` as formatted JSON.

#### Scenario: Reload saved progress
- **WHEN** a successful submission saves a stage and a later command loads the workspace
- **THEN** the completed stage slug is available from that workspace's configuration
