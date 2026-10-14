//! Markdown template resolution and terminal rendering

use regex::Regex;
use termimad::{MadSkin, Alignment, StyledChar};

/// Resolve the site's template blocks for a human reader with no fixed language
pub fn resolve_templates(text: &str) -> String {
    let mut text = text.to_string();

    // {{#reader_is_bot}}...{{/reader_is_bot}} - drop entirely
    text = Regex::new(r"(?s)\{\{#reader_is_bot\}\}.*?\{\{/reader_is_bot\}\}")
        .unwrap()
        .replace_all(&text, "")
        .into_owned();

    // {{#reader_is_human}}...{{/reader_is_human}} - keep content
    text = Regex::new(r"(?s)\{\{#reader_is_human\}\}(.*?)\{\{/reader_is_human\}\}")
        .unwrap()
        .replace_all(&text, "$1")
        .into_owned();

    // {{#lang_is_X}}...{{/lang_is_X}} - drop (single-language blocks)
    // Since regex doesn't support backreferences, we handle this by finding all
    // opening tags, capturing the language, then finding the matching closing tag.
    text = remove_lang_is_blocks(&text);

    // {{^lang_is_X}}...{{/lang_is_X}} - keep (inverse blocks, can be nested)
    // Repeat until no changes (handles nesting)
    text = remove_lang_not_blocks(&text);

    // {{language_name}} -> "your language"
    text = text.replace("{{language_name}}", "your language");

    // Collapse excessive newlines
    text = Regex::new(r"\n{3,}")
        .unwrap()
        .replace_all(&text, "\n\n")
        .into_owned();

    text.trim().to_string() + "\n"
}

/// Remove {{#lang_is_X}}...{{/lang_is_X}} blocks (drop them entirely)
fn remove_lang_is_blocks(text: &str) -> String {
    let open_regex = Regex::new(r"\{\{#lang_is_(\w+)\}\}").unwrap();
    let _close_regex = Regex::new(r"\{\{/lang_is_(\w+)\}\}").unwrap();

    let mut result = String::new();
    let mut pos = 0;

    while let Some(open_match) = open_regex.find_at(text, pos) {
        // Copy text before the opening tag
        result.push_str(&text[pos..open_match.start()]);

        // Extract the language from the opening tag
        if let Some(caps) = open_regex.captures_at(text, open_match.start()) {
            let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");

            // Find the matching closing tag
            let close_pattern = format!("{{{{/lang_is_{}}}}}", regex::escape(lang));
            let close_regex_specific = Regex::new(&close_pattern).unwrap();

            if let Some(close_match) = close_regex_specific.find_at(text, open_match.end()) {
                // Skip the entire block (opening tag + content + closing tag)
                pos = close_match.end();
            } else {
                // No matching closing tag, keep the opening tag and continue
                result.push_str(&text[open_match.start()..open_match.end()]);
                pos = open_match.end();
            }
        } else {
            pos = open_match.end();
        }
    }

    // Copy remaining text
    result.push_str(&text[pos..]);
    result
}

/// Remove {{^lang_is_X}}...{{/lang_is_X}} blocks (keep content, drop tags)
fn remove_lang_not_blocks(text: &str) -> String {
    let open_regex = Regex::new(r"\{\{\^lang_is_(\w+)\}\}").unwrap();

    let mut result = text.to_string();
    let mut changed = true;

    while changed {
        changed = false;
        let mut new_result = String::new();
        let mut pos = 0;

        while let Some(open_match) = open_regex.find_at(&result, pos) {
            new_result.push_str(&result[pos..open_match.start()]);

            if let Some(caps) = open_regex.captures_at(&result, open_match.start()) {
                let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let close_pattern = format!("{{{{/lang_is_{}}}}}", regex::escape(lang));
                let close_regex_specific = Regex::new(&close_pattern).unwrap();

                if let Some(close_match) = close_regex_specific.find_at(&result, open_match.end()) {
                    // Keep the content between tags, skip the tags themselves
                    new_result.push_str(&result[open_match.end()..close_match.start()]);
                    pos = close_match.end();
                    changed = true;
                } else {
                    // No matching closing tag
                    new_result.push_str(&result[open_match.start()..open_match.end()]);
                    pos = open_match.end();
                }
            } else {
                pos = open_match.end();
            }
        }

        new_result.push_str(&result[pos..]);
        result = new_result;
    }

    result
}

/// Create a MadSkin for terminal markdown rendering
pub fn create_skin(no_color: bool) -> MadSkin {
    let mut skin = MadSkin::default();

    if no_color {
        // No colors - use plain styles
        skin.headers[0].set_fg(termimad::rgb(255, 255, 255));
        skin.headers[1].set_fg(termimad::rgb(255, 255, 255));
        skin.headers[2].set_fg(termimad::rgb(200, 200, 200));
        skin.headers[3].set_fg(termimad::rgb(180, 180, 180));
        skin.bold.set_fg(termimad::rgb(255, 255, 255));
        skin.italic.set_fg(termimad::rgb(200, 200, 200));
        skin.code_block.align = Alignment::Left;
        skin.code_block.set_fg(termimad::rgb(150, 150, 150));
        skin.inline_code.set_fg(termimad::rgb(180, 180, 180));
        skin.quote_mark.set_fg(termimad::rgb(120, 120, 120));
    } else {
        // Nice color scheme
        skin.headers[0].set_fg(termimad::rgb(255, 180, 60));   // Orange
        skin.headers[1].set_fg(termimad::rgb(100, 200, 255));  // Blue
        skin.headers[2].set_fg(termimad::rgb(120, 220, 120));  // Green
        skin.headers[3].set_fg(termimad::rgb(180, 180, 255));  // Light purple
        skin.bold.set_fg(termimad::rgb(255, 220, 100));
        skin.italic.set_fg(termimad::rgb(200, 200, 255));
        skin.code_block.set_fg(termimad::rgb(160, 200, 160));
        skin.code_block.align = Alignment::Left;
        skin.inline_code.set_fg(termimad::rgb(255, 180, 80));
        skin.quote_mark.set_fg(termimad::rgb(100, 180, 100));
    }

    // Common settings
    skin.paragraph.align = Alignment::Left;
    skin.bullet = StyledChar::nude('•');
    skin.scrollbar.thumb.set_char('▐');
    skin.scrollbar.track.set_char(' ');

    skin
}

/// Render markdown to terminal using termimad
pub fn render_markdown(text: &str, _no_color: bool, _width: Option<usize>) -> String {
    let skin = create_skin(_no_color);

    let fmt_text = skin.term_text(text);
    fmt_text.to_string()
}

/// Render markdown as plain text (no formatting)
pub fn render_plain(text: &str) -> String {
    text.to_string()
}

/// Stage header rendering
pub fn render_stage_header(
    stage_num: usize,
    total: usize,
    name: &str,
    difficulty: &str,
    completed: bool,
    no_color: bool,
) -> String {
    use owo_colors::OwoColorize;

    let status = if completed { "✔ done" } else { "▶ current" };
    let status_styled = if no_color {
        status.to_string()
    } else if completed {
        status.green().to_string()
    } else {
        status.yellow().to_string()
    };

    let header = format!("Stage {}/{} · {} · {} {}", stage_num, total, name, difficulty, status_styled);
    let underline = "─".repeat(header.chars().count());

    if no_color {
        format!("{}\n{}\n", header, underline)
    } else {
        format!("{}\n{}\n", header.bold(), underline.dimmed())
    }
}

/// Progress bar style rendering for status command
pub fn render_progress_item(
    index: usize,
    _total: usize,
    stage: &crate::course::Stage,
    is_current: bool,
    is_completed: bool,
    no_color: bool,
) -> String {
    use owo_colors::OwoColorize;

    let num = format!("{:>3}.", index + 1);
    let name = &stage.name;
    let ext = stage.primary_extension_slug.as_ref().map(|e| format!(" [{}]", e)).unwrap_or_default();

    let (marker, styled) = if no_color {
        if is_completed { ("✔", format!("{} {}{}", "✔", num, name)) }
        else if is_current { ("▶", format!("{} {}{}", "▶", num, name)) }
        else { ("·", format!("{} {}{}", "·", num, name)) }
    } else if is_completed {
        ("✔", format!("{} {}{}", "✔".green(), num.dimmed(), name.dimmed()))
    } else if is_current {
        ("▶", format!("{} {}{}", "▶".yellow(), num.bold(), name.bold()))
    } else {
        ("·", format!("{} {}{}", "·".dimmed(), num.dimmed(), name.dimmed()))
    };

    let ext_styled = if no_color { ext } else { ext.dimmed().to_string() };

    format!(" {} {}{}", marker, styled, ext_styled)
}