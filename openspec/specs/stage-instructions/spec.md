# Stage Instructions

## Purpose

Present cached stage descriptions as readable terminal output or structured data,
and allow opening the original description in a local editor.

## Requirements

### Requirement: Resolve stage descriptions with fallbacks
The `show [stage]` command SHALL default to the current stage, use a matching
`stage_descriptions/*-<slug>.md` file when available, fall back to the stage's
`marketing_md`, and finally display `(no description found)` if both are absent.

#### Scenario: Prefer a cached description
- **WHEN** the selected stage has a matching Markdown file and marketing text
- **THEN** `show` presents the matching Markdown file's content

#### Scenario: Fall back when no description file exists
- **WHEN** no matching Markdown file exists and the stage has marketing text
- **THEN** `show` presents the marketing text

### Requirement: Resolve course templates for a human reader
Instruction rendering SHALL remove bot-only and positive language-specific
blocks, retain human-only and inverse language-specific content, replace
`{{language_name}}` with `your language`, and collapse excessive blank lines.

#### Scenario: Render language-neutral instructions
- **WHEN** a description contains human, bot, language-specific, and inverse
  language-specific template blocks
- **THEN** the resolved description retains human and inverse-block content
- **AND** it excludes bot and positive language-specific content

### Requirement: Support terminal, raw, and JSON instruction output
The `show` command SHALL offer a stage header and rendered Markdown by default,
resolved Markdown without a header or footer with `--raw`, and structured stage
data with `--json`; JSON output takes precedence over raw output.

#### Scenario: Request structured output
- **WHEN** `show --json` selects an enabled stage
- **THEN** stdout contains a JSON object with `stage`, `total`, `name`, `slug`,
  `difficulty`, `completed`, and resolved `markdown` fields
- **AND** no stage header or workflow footer is included

#### Scenario: Request raw Markdown
- **WHEN** `show --raw` runs without `--json`
- **THEN** it emits the resolved Markdown without terminal rendering, a stage
  header, or a workflow footer

### Requirement: Open original stage Markdown in an editor
The `open [stage]` command SHALL select the current or requested stage and open
its matching Markdown file using `EDITOR`, defaulting to `vi`.

#### Scenario: Open a stage description
- **WHEN** the stage has a matching Markdown file and `EDITOR` is set
- **THEN** the configured editor executable is launched with that file path

#### Scenario: Description file is absent
- **WHEN** the selected stage has no matching Markdown file
- **THEN** `open` reports an error rather than opening fallback marketing text
