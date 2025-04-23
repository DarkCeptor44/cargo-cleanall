//! # cargo-cleanall
//!
//! Cleans all project builds (`target` directory) by running `cargo clean` in all directories that have a `Cargo.toml` file in them. The search is not recursive and will only clean the top level directories in the path.
//!
//! ## Getting Started
//!
//! The `cargo-cleanall` CLI can be installed with the following command:
//!
//! ```bash
//! cargo install --git https://github.com/DarkCeptor44/cargo-cleanall
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! $ cargo cleanall -h
//! Cleans all project builds
//!
//! Usage: cargo-cleanall.exe cleanall [OPTIONS] [PATH]
//!
//! Arguments:
//!   [PATH]  Path where the Rust projects are [default: .]
//!
//! Options:
//!       --dry-run  Dry run
//!   -q, --quiet    Quiet
//!   -h, --help     Print help
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use anyhow::Result;
use cargo_cleanall::clean_all;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Cleans all project builds")]
    Cleanall {
        #[arg(help = "Path where the Rust projects are", default_value = ".")]
        path: String,

        #[arg(long, help = "Dry run", default_value_t)]
        dry_run: bool,

        #[arg(short, long, help = "Quiet", default_value_t)]
        quiet: bool,
    },
}

fn main() -> Result<()> {
    let args = App::parse();

    match args.command {
        Commands::Cleanall {
            path,
            dry_run,
            quiet,
        } => clean_all(path, dry_run, !quiet)?,
    }

    Ok(())
}
