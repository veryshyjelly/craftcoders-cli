//! cc - Practice CodeCrafters challenges locally

mod cli;
mod commands;
mod config;
mod course;
mod git_ops;
mod project;
mod render;
mod tester;

use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;
use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .init();

    // Disable colors if requested (via flag or NO_COLOR env var)
    let no_color = cli.no_color || std::env::var("NO_COLOR").is_ok();
    if no_color {
        owo_colors::set_override(false);
    }

    let cc_home = cli.cc_home.as_deref();

    let result = match cli.command {
        Commands::Init(args) => commands::cmd_init(&args, cc_home, no_color),
        Commands::Status => commands::cmd_status(&(), cc_home, no_color),
        Commands::Show(args) => commands::cmd_show(&args, cc_home, no_color),
        Commands::Test(args) => {
            let rc = commands::cmd_test(&args, cc_home, no_color)?;
            std::process::exit(rc);
        }
        Commands::Submit(args) => commands::cmd_submit(&args, cc_home, no_color),
        Commands::Ext(args) => commands::cmd_ext(&args.action, cc_home, no_color),
        Commands::Update => commands::cmd_update(&(), cc_home, no_color),
        Commands::Doctor => commands::cmd_doctor(&(), cc_home, no_color),
        Commands::List => commands::cmd_list(&(), cc_home, no_color),
        Commands::Open(args) => commands::cmd_open(&args, cc_home, no_color),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        // Print error chain
        for cause in e.chain().skip(1) {
            eprintln!("  caused by: {}", cause);
        }
        std::process::exit(1);
    }

    Ok(())
}