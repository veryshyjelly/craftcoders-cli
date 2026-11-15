# Course Extensions

## Purpose

Enable optional stage groups already declared in a cached course definition and
persist their selection in each workspace without downloading extension repos.

## Requirements

### Requirement: List declared course extensions
The `ext list` command SHALL list extensions from the course definition with
their slug, name, enabled status, and associated stage count.

#### Scenario: Inspect available extensions
- **WHEN** the definition declares enabled and disabled extensions
- **THEN** `ext list` displays each extension with its corresponding status and count

### Requirement: Enable only known extensions without duplicates
The `ext add <slug>` command SHALL accept only extensions declared by the course,
append a newly enabled slug to the saved extension list, and avoid duplicating an
already enabled slug.

#### Scenario: Enable a declared extension
- **WHEN** the requested slug is declared but not yet enabled
- **THEN** it is appended to `.craftcoders.json` and its stages enter the enabled order

#### Scenario: Enable the same extension twice
- **WHEN** the requested slug is already enabled
- **THEN** the saved extension list retains a single copy of that slug

#### Scenario: Reject an unknown extension
- **WHEN** the requested slug does not appear in the course definition
- **THEN** the CLI reports an unknown extension and lists valid slugs
- **AND** the saved extension list remains unchanged

### Requirement: Disable an extension while retaining completion history
The `ext remove <slug>` command SHALL remove that slug from the saved extension
list without deleting completed stage slugs.

#### Scenario: Disable and later re-enable a completed extension
- **WHEN** an extension with completed stages is removed and then re-enabled
- **THEN** its stages re-enter the enabled order with their completion history intact

#### Scenario: Remove an extension that is not enabled
- **WHEN** the requested slug is absent from the saved extension list
- **THEN** the command succeeds with the enabled extension list unchanged
