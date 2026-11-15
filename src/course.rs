//! Course definition loading and stage management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseDefinition {
    pub name: String,
    pub slug: String,
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stage {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub primary_extension_slug: Option<String>,
    #[serde(default)]
    pub marketing_md: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extension {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// Load course definition from course-definition.yml
pub fn load_definition(definition_dir: &Path) -> Result<CourseDefinition> {
    let path = definition_dir.join("course-definition.yml");
    let content =
        fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let def: CourseDefinition =
        serde_yaml::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;
    Ok(def)
}

/// Get ordered stages for a course with enabled extensions
pub fn ordered_stages<'a>(
    def: &'a CourseDefinition,
    enabled_extensions: &[String],
) -> Vec<&'a Stage> {
    let mut stages: Vec<&'a Stage> = def
        .stages
        .iter()
        .filter(|s| s.primary_extension_slug.is_none())
        .collect();

    let ext_map: HashMap<_, _> = def
        .extensions
        .iter()
        .map(|e| (e.slug.as_str(), e))
        .collect();

    for ext_slug in enabled_extensions {
        if let Some(_ext) = ext_map.get(ext_slug.as_str()) {
            stages.extend(
                def.stages
                    .iter()
                    .filter(|s| s.primary_extension_slug.as_deref() == Some(ext_slug.as_str())),
            );
        }
    }
    stages
}

/// Find stage by number (1-indexed) or slug
pub fn find_stage<'a>(stages: &'a [&Stage], query: &str) -> Option<&'a Stage> {
    match query.parse::<usize>() {
        Ok(num) if num > 0 && num <= stages.len() => Some(stages[num - 1]),
        _ => stages.iter().find(|s| s.slug == query).copied(),
    }
}

/// Get stage index (0-based) for current progress
pub fn current_stage_index(stages: &[&Stage], completed: &[String]) -> Option<usize> {
    let completed_set: std::collections::HashSet<_> = completed.iter().collect();
    stages.iter().position(|s| !completed_set.contains(&s.slug))
}

/// Read stage markdown description
pub fn read_stage_markdown(definition_dir: &Path, slug: &str) -> Result<Option<String>> {
    let desc_dir = definition_dir.join("stage_descriptions");
    if !desc_dir.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(&desc_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.ends_with(&format!("-{}.md", slug)) {
            let content = fs::read_to_string(entry.path())?;
            return Ok(Some(content));
        }
    }
    Ok(None)
}

/// Extension info for listing
pub struct ExtensionInfo {
    pub slug: String,
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub stage_count: usize,
    pub enabled: bool,
}

pub fn list_extensions(def: &CourseDefinition, enabled: &[String]) -> Vec<ExtensionInfo> {
    let enabled_set: std::collections::HashSet<_> = enabled.iter().collect();
    def.extensions
        .iter()
        .map(|e| {
            let count = def
                .stages
                .iter()
                .filter(|s| s.primary_extension_slug.as_deref() == Some(&e.slug))
                .count();
            ExtensionInfo {
                slug: e.slug.clone(),
                name: e.name.clone(),
                description: e.description.clone(),
                stage_count: count,
                enabled: enabled_set.contains(&e.slug),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(slug: &str, extension: Option<&str>) -> Stage {
        Stage {
            name: slug.into(),
            slug: slug.into(),
            difficulty: String::new(),
            primary_extension_slug: extension.map(String::from),
            marketing_md: None,
        }
    }

    #[test]
    fn enabled_extensions_follow_saved_order_after_base_stages() {
        let definition = CourseDefinition {
            name: "Demo".into(),
            slug: "demo".into(),
            stages: vec![
                stage("a-one", Some("a")),
                stage("base-one", None),
                stage("b-one", Some("b")),
                stage("base-two", None),
                stage("a-two", Some("a")),
                stage("disabled", Some("c")),
            ],
            extensions: ["a", "b", "c"]
                .into_iter()
                .map(|slug| Extension {
                    slug: slug.into(),
                    name: slug.into(),
                    description: String::new(),
                })
                .collect(),
        };
        let stages = ordered_stages(&definition, &["b".into(), "a".into()]);
        let slugs: Vec<_> = stages.iter().map(|stage| stage.slug.as_str()).collect();
        assert_eq!(slugs, ["base-one", "base-two", "b-one", "a-one", "a-two"]);
    }

    #[test]
    fn selectors_use_one_based_numbers_and_preserve_exact_slug_fallback() {
        let zero = stage("0", None);
        let other = stage("other", None);
        let stages = [&zero, &other];
        assert_eq!(find_stage(&stages, "1").unwrap().slug, "0");
        assert_eq!(find_stage(&stages, "2").unwrap().slug, "other");
        assert_eq!(find_stage(&stages, "0").unwrap().slug, "0");
        assert_eq!(find_stage(&stages, "other").unwrap().slug, "other");
        assert!(find_stage(&stages, "3").is_none());
    }

    #[test]
    fn completion_gaps_select_the_first_unfinished_enabled_stage() {
        let one = stage("one", None);
        let two = stage("two", None);
        let three = stage("three", None);
        let stages = [&one, &two, &three];
        assert_eq!(
            current_stage_index(&stages, &["one".into(), "three".into()]),
            Some(1)
        );
        assert_eq!(
            current_stage_index(&stages, &["three".into(), "one".into(), "two".into()]),
            None
        );
    }
}
