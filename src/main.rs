//! craftcoders - Practice CodeCrafters challenges locally

// Keep CLI-owned writes behind one policy, including explicit owo-colors styles.
// Child processes inherit the real streams and own their output policy.
macro_rules! print {
    ($($arg:tt)*) => { crate::output::print(format_args!($($arg)*), false) };
}
macro_rules! println {
    () => { crate::output::print(format_args!("\n"), false) };
    ($($arg:tt)*) => { crate::output::print(format_args!("{}\n", format_args!($($arg)*)), false) };
}
macro_rules! eprintln {
    ($($arg:tt)*) => { crate::output::print(format_args!("{}\n", format_args!($($arg)*)), true) };
}

mod cli;
mod commands;
mod config;
mod course;
mod git_ops;
mod output;
mod project;
mod render;
mod tester;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use owo_colors::OwoColorize;
use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    // Configure clap before parsing so help and argument errors follow the policy.
    let no_color = std::env::args_os().any(|arg| arg == "--no-color")
        || std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
    output::set_no_color(no_color);
    let command = Cli::command();
    let command = if no_color {
        command.color(clap::ColorChoice::Never)
    } else {
        command
    };
    let matches = command.get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    fmt()
        .with_env_filter(filter)
        .with_ansi(!no_color)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .init();

    // Disable colors if requested (via flag or NO_COLOR env var)
    let no_color = cli.no_color || no_color;
    if no_color {
        owo_colors::set_override(false);
    }

    let cc_home = cli.cc_home.as_deref();

    let result = match cli.command {
        Commands::Init(args) => commands::cmd_init(&args, cc_home, no_color),
        Commands::Status => commands::cmd_status(&(), cc_home, no_color),
        Commands::Show(args) => commands::cmd_show(&args, cc_home, no_color),
        Commands::Test(args) => match commands::cmd_test(&args, cc_home, no_color) {
            Ok(rc) => std::process::exit(rc),
            Err(error) => Err(error),
        },
        Commands::Submit(args) => commands::cmd_submit(&args, cc_home, no_color),
        Commands::Ext(args) => commands::cmd_ext(&args.action, cc_home, no_color),
        Commands::Update(args) => commands::cmd_update(&args, cc_home, no_color),
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
