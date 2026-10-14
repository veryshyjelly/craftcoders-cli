//! CLI argument definitions

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "craftcoders",
    version,
    about = "Practice CodeCrafters challenges locally — no website, no starters, just you and the official testers",
    long_about = None,
    propagate_version = true,
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Set the base directory for courses and testers (default: ~/.craftcodeers)
    #[arg(long, global = true, env = "CC_HOME")]
    pub cc_home: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub no_color: bool,

    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project for a course
    Init(InitArgs),

    /// Show progress through the course
    Status,

    /// Show instructions for a stage (default: current stage)
    Show(ShowArgs),

    /// Run tests for previous stages + current stage
    Test(TestArgs),

    /// Run tests and advance to next stage on pass
    Submit(SubmitArgs),

    /// Manage course extensions
    Ext(ExtArgs),

    /// Update course and tester to latest, rebuild tester
    Update,

    /// Check environment (git, go, docker, runtime deps)
    Doctor,

    /// List available courses
    List,

    /// Open current stage in $EDITOR
    Open(OpenArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Course slug (e.g. redis, git, sqlite, shell, kafka)
    pub course: String,

    /// Project directory (default: my-<course>)
    pub dir: Option<PathBuf>,

    /// Override tester repo name (default: <course>-tester)
    #[arg(long)]
    pub tester_repo: Option<String>,

    /// Skip tester build
    #[arg(long)]
    pub no_build: bool,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Stage number or slug (default: current)
    pub stage: Option<String>,

    /// Raw markdown without terminal rendering
    #[arg(long)]
    pub raw: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Run tester inside a Docker image (project mounted at /app)
    #[arg(long, value_name = "IMAGE")]
    pub docker: Option<String>,

    /// Print what would run without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Run only the current stage (skip previous stages)
    #[arg(long)]
    pub only_current: bool,

    /// Run up to a specific stage (number or slug)
    #[arg(long, value_name = "STAGE")]
    pub upto: Option<String>,
}

#[derive(Args, Debug)]
pub struct SubmitArgs {
    /// Run tester inside a Docker image (project mounted at /app)
    #[arg(long, value_name = "IMAGE")]
    pub docker: Option<String>,

    /// Print what would run without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Run up to a specific stage (number or slug) and submit if passes
    #[arg(long, value_name = "STAGE")]
    pub upto: Option<String>,
}

#[derive(Args, Debug)]
pub struct ExtArgs {
    #[command(subcommand)]
    pub action: ExtAction,
}

#[derive(Subcommand, Debug)]
pub enum ExtAction {
    /// List available extensions
    List,
    /// Enable an extension
    Add { slug: String },
    /// Disable an extension
    Remove { slug: String },
}

#[derive(Args, Debug)]
pub struct OpenArgs {
    /// Stage to open (default: current)
    pub stage: Option<String>,
}