# terminal-output Specification

## Purpose

Provide predictable terminal output for users who disable color in local CLI workflows and diagnostics.

## Requirements

### Requirement: Honor global color suppression
The CLI SHALL omit ANSI styling from its own stdout and stderr when `--no-color` is supplied or `NO_COLOR` is nonempty, including status, instructions, diagnostics, previews, and errors. Child process output is outside this guarantee.

#### Scenario: Disable color with a flag
- **WHEN** a command runs with `--no-color`
- **THEN** its CLI-owned output contains no ANSI escape sequences, including errors

#### Scenario: Disable color through the environment
- **WHEN** a command runs with nonempty `NO_COLOR`
- **THEN** CLI-owned stdout and stderr omit ANSI styling
- **AND** structured JSON and raw Markdown remain usable in their existing formats
