# Course Stages

## Purpose

Interpret cached course definitions to provide a consistent enabled stage order,
stage lookup, and current progress for challenge commands.

## Requirements

### Requirement: Load and order enabled stages
The CLI SHALL load `course-definition.yml`, place base stages in definition order
first, and append each enabled extension's stages in the saved extension order,
preserving definition order within each extension.

#### Scenario: Include enabled extensions
- **WHEN** a course has base stages and the saved extensions are `b` followed by `a`
- **THEN** the enabled stage order is base stages, extension `b` stages, then
  extension `a` stages
- **AND** stages belonging to disabled extensions are excluded

### Requirement: Select a stage by number or slug
Stage selectors SHALL resolve a valid one-based number or an exact enabled stage
slug and report an error when no matching stage is found.

#### Scenario: Select the second enabled stage
- **WHEN** a stage selector is `2` and at least two stages are enabled
- **THEN** the second stage in the enabled order is selected

#### Scenario: Reject an unavailable stage
- **WHEN** a selector matches neither a valid stage number nor an enabled slug
- **THEN** the command reports that no stage was found

### Requirement: Derive the current stage from completion slugs
The current stage SHALL be the first enabled stage whose slug is absent from
`completed`, with no current stage when every enabled stage is complete.

#### Scenario: Completion contains a gap
- **WHEN** the first and third stages are complete and the second is not
- **THEN** the second stage is current

#### Scenario: Finish the enabled course
- **WHEN** all enabled stage slugs are in the completed list
- **THEN** no current stage is selected
- **AND** `status` reports that all stages are complete

### Requirement: Display saved course progress
The `status` command SHALL show the course name, a completion count, and each
enabled stage with its completed or current status.

#### Scenario: Inspect an in-progress workspace
- **WHEN** `status` runs with completed stages and an unfinished current stage
- **THEN** it lists the enabled stages in order and distinguishes completed
  stages from the current stage
