//! Command implementations

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use which::which;

use crate::cli::{ExtAction, InitArgs, OpenArgs, ShowArgs, SubmitArgs, TestArgs};
use crate::config::{cc_home, find_project_root, CoursePaths};
use crate::course::{current_stage_index, find_stage, list_extensions, load_definition, ordered_stages, read_stage_markdown};
use crate::git_ops::{ensure_repos, update_repos};
use crate::project::{init_project, load_project, save_project};
use crate::render::{render_markdown, render_plain, render_progress_item, render_stage_header, resolve_templates};
use crate::tester::{build_tester, build_test_cases, run_tester};

/// Initialize a new project
pub fn cmd_init(args: &InitArgs, cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    let cc_home_dir = cc_home(cli_cc_home)?;
    let (def_dir, _tester_dir) = ensure_repos(&cc_home_dir, &args.course, args.tester_repo.as_deref())?;

    let course_paths = CoursePaths::new(&cc_home_dir, &args.course);

    if !args.no_build {
        build_tester(&course_paths, &args.course, which("go").is_err())?;
    }

    let target = args.dir.clone().unwrap_or_else(|| Path::new(&format!("my-{}", args.course)).to_path_buf());
    let target = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());

    if target.exists() && target.read_dir()?.next().is_some() {
        anyhow::bail!("{} already exists and is not empty", target.display());
    }

    let def = load_definition(&def_dir)?;
    init_project(&target, args.course.clone())?;

    let base_stages = ordered_stages(&def, &[]).len();

    println!("{} {}", "Created".green().bold(), target.display());
    println!("{}: {} base stages.", def.name.bold(), base_stages.to_string().bold());
    println!("Next:  cd {} && craftcoders show", target.display());

    Ok(())
}

/// Show project status
pub fn cmd_status(_args: &(), cli_cc_home: Option<&Path>, no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    let stages = ordered_stages(&def, &config.extensions);
    let cur_idx = current_stage_index(&stages, &config.completed);

    println!("{}   ({}/{})", def.name.bold(), config.completed.len().to_string().bold(), stages.len());

    for (i, stage) in stages.iter().enumerate() {
        let is_current = cur_idx == Some(i);
        let is_completed = config.completed.contains(&stage.slug);
        println!("{}", render_progress_item(i, stages.len(), stage, is_current, is_completed, no_color));
    }

    if cur_idx.is_none() {
        println!("{}", "All stages complete.".green().bold());
    }

    Ok(())
}

/// Show stage instructions
pub fn cmd_show(args: &ShowArgs, cli_cc_home: Option<&Path>, no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    let stages = ordered_stages(&def, &config.extensions);
    let cur_idx = current_stage_index(&stages, &config.completed);

    let stage_idx = if let Some(ref q) = args.stage {
        find_stage(&stages, q).map(|s| stages.iter().position(|st| st.slug == s.slug).unwrap())
    } else {
        cur_idx
    }.context("no stage found")?;

    let stage = stages[stage_idx];
    let is_completed = config.completed.contains(&stage.slug);

    // Get and render markdown
    let markdown = read_stage_markdown(&course_paths.definition, &stage.slug)?
        .or_else(|| stage.marketing_md.clone())
        .unwrap_or_else(|| "(no description found)".to_string());

    let resolved = resolve_templates(&markdown);

    if args.json {
        println!("{}", serde_json::json!({
            "stage": stage_idx + 1,
            "total": stages.len(),
            "name": stage.name,
            "slug": stage.slug,
            "difficulty": stage.difficulty,
            "completed": is_completed,
            "markdown": resolved,
        }));
        return Ok(());
    }

    // Print header (not for raw mode)
    if !args.raw {
        print!("{}", render_stage_header(stage_idx + 1, stages.len(), &stage.name, &stage.difficulty, is_completed, no_color));
    }

    if args.raw {
        print!("{}", render_plain(&resolved));
    } else {
        print!("{}", render_markdown(&resolved, no_color, None));
    }

    if !args.raw {
        println!("\n{}", "When you think it works:  craftcoders test   |   to advance:  craftcoders submit".dimmed());
    }

    Ok(())
}

/// Run tests - common logic for test and submit
fn run_test_common(
    docker_image: Option<&str>,
    dry_run: bool,
    only_current: bool,
    upto: Option<&str>,
    cli_cc_home: Option<&Path>,
    no_color: bool,
) -> Result<i32> {
    let project_root = find_project_root(Path::new("."))?;
    let config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    let stages = ordered_stages(&def, &config.extensions);
    let cur_idx = current_stage_index(&stages, &config.completed)
        .context("all stages are complete. Enable more with `craftcoders ext add <slug>`.")?;

    // Determine target index
    let target_idx = if let Some(q) = upto {
        find_stage(&stages, q).map(|s| stages.iter().position(|st| st.slug == s.slug).unwrap())
    } else {
        Some(cur_idx)
    }.context("stage not found")?;

    let test_cases = build_test_cases(&stages, target_idx, only_current);

    println!("{}", format!("Running {} stage(s) up to: {}", test_cases.len(), stages[target_idx].name).bold());

    let rc = run_tester(&project_root, &course_paths.tester_binary, &test_cases, docker_image, dry_run)?;

    Ok(rc)
}

/// Run tests
pub fn cmd_test(args: &TestArgs, cli_cc_home: Option<&Path>, no_color: bool) -> Result<i32> {
    run_test_common(args.docker.as_deref(), args.dry_run, args.only_current, args.upto.as_deref(), cli_cc_home, no_color)
}

/// Submit (test and advance on pass)
pub fn cmd_submit(args: &SubmitArgs, cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let mut config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    let stages = ordered_stages(&def, &config.extensions);
    let cur_idx = current_stage_index(&stages, &config.completed)
        .context("all stages are complete. Enable more with `craftcoders ext add <slug>`.")?;

    // Determine target index
    let target_idx = if let Some(ref q) = args.upto {
        find_stage(&stages, q).map(|s| stages.iter().position(|st| st.slug == s.slug).unwrap())
    } else {
        Some(cur_idx)
    }.context("stage not found")?;

    let test_cases = build_test_cases(&stages, target_idx, false);

    println!("{}", format!("Running {} stage(s) up to: {}", test_cases.len(), stages[target_idx].name).bold());

    let rc = run_tester(&project_root, &course_paths.tester_binary, &test_cases, args.docker.as_deref(), args.dry_run)?;

    if args.dry_run {
        return Ok(());
    }

    if rc != 0 {
        println!("\n{}", "Not passing yet. Keep going, then run `craftcoders submit` again.".red().bold());
        std::process::exit(rc);
    }

    // Advance progress
    config.completed.push(stages[target_idx].slug.clone());
    save_project(&project_root, &config)?;

    println!("\n{} {}", "✔ Stage".green().bold(), format!("{} complete: {}", target_idx + 1, stages[target_idx].name).green());

    if target_idx + 1 < stages.len() {
        println!("Next: stage {} - {}   (run `craftcoders show`)", target_idx + 2, stages[target_idx + 1].name);
    } else {
        println!("{}", "That was the last enabled stage. Add more with `craftcoders ext add <slug>`.".green());
    }

    Ok(())
}

/// Manage extensions
pub fn cmd_ext(action: &ExtAction, cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let mut config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    match action {
        ExtAction::List => {
            let exts = list_extensions(&def, &config.extensions);
            for ext in exts {
                let status = if ext.enabled {
                    format!("{}", "on".green())
                } else {
                    format!("{}", "off".dimmed())
                };
                let count = ext.stage_count.to_string();
                println!(" {}  {:<20} {} ({} stages)", status, ext.slug.bold(), ext.name, count);
            }
        }
        ExtAction::Add { slug } => {
            if !def.extensions.iter().any(|e| e.slug == *slug) {
                anyhow::bail!("unknown extension; choose from: {}", def.extensions.iter().map(|e| e.slug.as_str()).collect::<Vec<_>>().join(", "));
            }
            if !config.extensions.contains(slug) {
                config.extensions.push(slug.clone());
                save_project(&project_root, &config)?;
            }
            println!("extensions enabled: {}", config.extensions.join(", "));
        }
        ExtAction::Remove { slug } => {
            config.extensions.retain(|s| s != slug);
            save_project(&project_root, &config)?;
            println!("extensions enabled: {}", config.extensions.join(", "));
        }
    }

    Ok(())
}

/// Update course and tester
pub fn cmd_update(_args: &(), cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);

    update_repos(&cc_home_dir, &config.course)?;
    build_tester(&course_paths, &config.course, which("go").is_err())?;

    println!("{}", "updated course and tester".green().bold());
    Ok(())
}

/// Doctor - check environment
pub fn cmd_doctor(_args: &(), cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    println!("{}", "Environment Check".bold().underline());
    println!();

    // Git
    match which("git") {
        Ok(path) => println!("{} git: {}", "✔".green(), path.display()),
        Err(_) => println!("{} git: {}", "✘".red(), "not found (required)"),
    }

    // Go
    match which("go") {
        Ok(_path) => {
            let out = Command::new("go").arg("version").output()?;
            let version = String::from_utf8_lossy(&out.stdout);
            println!("{} go: {}", "✔".green(), version.trim());
        }
        Err(_) => println!("{} go: {}", "⚠".yellow(), "not found (will use Docker for tester build)"),
    }

    // Docker
    match which("docker") {
        Ok(_path) => {
            let out = Command::new("docker").arg("--version").output()?;
            let version = String::from_utf8_lossy(&out.stdout);
            println!("{} docker: {}", "✔".green(), version.trim());
        }
        Err(_) => println!("{} docker: {}", "⚠".yellow(), "not found (required for --docker and tester build without Go)"),
    }

    // CC_HOME
    let cc_home_dir = cc_home(cli_cc_home)?;
    println!("{} CC_HOME: {}", "ℹ".blue(), cc_home_dir.display());

    // Project
    match find_project_root(Path::new(".")) {
        Ok(root) => {
            let config = load_project(&root)?;
            println!("{} Project: {} ({})", "✔".green(), root.display(), config.course);
        }
        Err(_) => println!("{} Project: {}", "ℹ".blue(), "not in a project (run `craftcoders init <course>`)"),
    }

    // Check installed courses
    let courses_dir = cc_home_dir.join("courses");
    if courses_dir.exists() {
        let courses: Vec<_> = fs::read_dir(&courses_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        if !courses.is_empty() {
            println!("{} Installed courses: {}", "ℹ".blue(), courses.join(", "));
        }
    }

    println!();
    println!("{}", "For course-specific runtime dependencies, check the tester repo's README.".dimmed());

    Ok(())
}

/// List available courses (from known list)
pub fn cmd_list(_args: &(), cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    // Known courses - could be fetched from GitHub API but keep offline
    let known = [
        ("redis", "Build your own Redis"),
        ("git", "Build your own Git"),
        ("sqlite", "Build your own SQLite"),
        ("shell", "Build your own Shell"),
        ("kafka", "Build your own Kafka"),
        ("docker", "Build your own Docker"),
        ("http-server", "Build your own HTTP Server"),
        ("dns-server", "Build your own DNS Server"),
        ("bitcoin", "Build your own Bitcoin"),
        ("bittorrent", "Build your own BitTorrent"),
    ];

    println!("{}", "Available CodeCrafters Courses".bold().underline());
    println!();

    let cc_home_dir = cc_home(cli_cc_home)?;
    let courses_dir = cc_home_dir.join("courses");
    let installed: std::collections::HashSet<_> = if courses_dir.exists() {
        fs::read_dir(&courses_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    } else {
        Default::default()
    };

    for (slug, name) in known {
        let installed_marker = if installed.contains(slug) {
            format!(" {}", "✔".green())
        } else {
            String::new()
        };
        println!("  {}{}  {}", slug.bold(), installed_marker, name.dimmed());
    }

    println!();
    println!("{}", "Run `craftcoders init <slug>` to start a course.".dimmed());

    Ok(())
}

/// Open current stage in $EDITOR
pub fn cmd_open(args: &OpenArgs, cli_cc_home: Option<&Path>, _no_color: bool) -> Result<()> {
    let project_root = find_project_root(Path::new("."))?;
    let config = load_project(&project_root)?;
    let cc_home_dir = cc_home(cli_cc_home)?;
    let course_paths = CoursePaths::new(&cc_home_dir, &config.course);
    let def = load_definition(&course_paths.definition)?;

    let stages = ordered_stages(&def, &config.extensions);
    let cur_idx = current_stage_index(&stages, &config.completed);

    let stage_idx = if let Some(ref q) = args.stage {
        find_stage(&stages, q).map(|s| stages.iter().position(|st| st.slug == s.slug).unwrap())
    } else {
        cur_idx
    }.context("no stage found")?;

    let stage = stages[stage_idx];

    // Find the stage markdown file
    let desc_dir = course_paths.definition.join("stage_descriptions");
    let markdown_path = fs::read_dir(&desc_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with(&format!("-{}.md", stage.slug)))
        .map(|e| e.path());

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    if let Some(path) = markdown_path {
        let status = Command::new(&editor).arg(&path).status()?;
        if !status.success() {
            anyhow::bail!("editor exited with error");
        }
    } else {
        anyhow::bail!("no markdown file found for stage {}", stage.slug);
    }

    Ok(())
}