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
//!   -f, --fast           Fast mode (removes target directory directly, not recommended)
//!       --dry-run        Dry run
//!   -l, --limit <LIMIT>  Concurrency limit [default: 64]
//!   -h, --help           Print help
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use anyhow::{Context, Result, anyhow};
use cargo_cleanall::{clean_dir, get_cargo_projects};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::PathBuf, process::exit, sync::Arc};
use tokio::{fs::metadata, sync::Semaphore, task::JoinSet};

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
        path: PathBuf,

        #[arg(
            short,
            long,
            help = "Fast mode (removes target directory directly, not recommended)",
            default_value_t
        )]
        fast: bool,

        #[arg(long, help = "Dry run", default_value_t)]
        dry_run: bool,

        #[arg(short, long, help = "Concurrency limit", default_value_t = concurrency_limit())]
        limit: usize,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = main_impl().await {
        eprintln!("{}", format!("cargo-cleanall: {e:?}").red());
        exit(1);
    }
}

async fn main_impl() -> Result<()> {
    let args = App::parse();

    match args.command {
        Commands::Cleanall {
            path,
            fast,
            dry_run,
            limit,
        } => clean_all(path, fast, dry_run, limit).await?,
    }

    Ok(())
}

async fn clean_all(root: PathBuf, fast: bool, dry_run: bool, limit: usize) -> Result<()> {
    let meta = metadata(&root).await.context("failed to get metadata")?;
    if !meta.is_dir() {
        return Err(anyhow!("path is not a directory: {}", root.display()));
    }

    if limit == 0 {
        return Err(anyhow!("Concurrency limit must be greater than 0"));
    }

    let initial_paths = get_cargo_projects(&root)
        .await
        .context("failed to get Cargo projects")?;

    let semaphore = Arc::new(Semaphore::new(limit));
    let mut set: JoinSet<Result<Option<PathBuf>>> = JoinSet::new();
    for path in initial_paths {
        let semaphore = semaphore.clone();
        set.spawn(async move {
            let _permit = semaphore.acquire_owned().await?;
            let target_path = path.join("target");
            let meta = metadata(&target_path).await?;

            if meta.is_dir() {
                Ok(Some(path))
            } else {
                Ok(None)
            }
        });
    }

    let mut paths = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(Ok(Some(p))) = res {
            paths.push(p);
        }
    }

    let paths_len = paths.len();
    let pb = ProgressBar::new(paths_len.try_into()?);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed:.green}] [{bar:40.green/lime}] {pos}/{len} {eta}",
            )?
            .progress_chars("=> "),
    );

    let semaphore = Arc::new(Semaphore::new(limit));
    let mut set: JoinSet<Result<(PathBuf, bool)>> = JoinSet::new();

    for path in paths {
        let semaphore = semaphore.clone();

        set.spawn(async move {
            let _permit = semaphore.acquire_owned().await?;
            let result = clean_dir(&path, fast, dry_run).await?;

            Ok((path, result))
        });
    }

    let mut count: usize = 0;
    while let Some(res) = set.join_next().await {
        pb.inc(1);

        match res? {
            Ok((p, true)) => {
                count += 1;
                if dry_run {
                    pb.println(format!("{} {}", "Would clean:".yellow(), p.display()));
                }
            }
            Ok((_, false)) => (),
            Err(e) => pb.println(format!("{}", format!("Cleaning error: {e:?}\n").red())),
        }
    }

    pb.finish_and_clear();
    println!(
        "Cleaned {}/{} projects",
        count.to_string().cyan(),
        paths_len.to_string().cyan()
    );
    Ok(())
}

fn concurrency_limit() -> usize {
    let cpus = num_cpus::get();
    match cpus {
        0..=2 => cpus,
        3..=8 => cpus * 4,
        _ => (cpus * 4).min(128),
    }
}
