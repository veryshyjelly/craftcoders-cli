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
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let def: CourseDefinition = serde_yaml::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(def)
}

/// Get ordered stages for a course with enabled extensions
pub fn ordered_stages<'a>(def: &'a CourseDefinition, enabled_extensions: &[String]) -> Vec<&'a Stage> {
    let mut stages: Vec<&'a Stage> = def.stages.iter().filter(|s| s.primary_extension_slug.is_none()).collect();

    let ext_map: HashMap<_, _> = def.extensions.iter().map(|e| (e.slug.as_str(), e)).collect();

    for ext_slug in enabled_extensions {
        if let Some(_ext) = ext_map.get(ext_slug.as_str()) {
            stages.extend(def.stages.iter().filter(|s| s.primary_extension_slug.as_deref() == Some(ext_slug.as_str())));
        }
    }
    stages
}

/// Find stage by number (1-indexed) or slug
pub fn find_stage<'a>(stages: &'a [&Stage], query: &str) -> Option<&'a Stage> {
    if let Ok(num) = query.parse::<usize>() {
        if num > 0 && num <= stages.len() {
            return Some(stages[num - 1]);
        }
    }
    stages.iter().find(|s| s.slug == query).copied()
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
    pub description: String,
    pub stage_count: usize,
    pub enabled: bool,
}

pub fn list_extensions(def: &CourseDefinition, enabled: &[String]) -> Vec<ExtensionInfo> {
    let enabled_set: std::collections::HashSet<_> = enabled.iter().collect();
    def.extensions.iter().map(|e| {
        let count = def.stages.iter().filter(|s| s.primary_extension_slug.as_deref() == Some(&e.slug)).count();
        ExtensionInfo {
            slug: e.slug.clone(),
            name: e.name.clone(),
            description: e.description.clone(),
            stage_count: count,
            enabled: enabled_set.contains(&e.slug),
        }
    }).collect()
}