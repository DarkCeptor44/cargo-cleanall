//! # cargo-cleanall
//!
//! Provides a way to get a list of directories in a path that have a `Cargo.toml` file in them. The search is not recursive and will only clean the top level directories in the path.
//!
//! ## Getting Started
//!
//! Run `cargo add cargo-cleanall` or add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargo-cleanall = "^1"
//! ```
//!
//! ## Library Usage
//!
//! ```rust,no_run
//! use cargo_cleanall::get_cargo_projects;
//!
//! let (dirs, elapsed) = get_cargo_projects("path/to/directory").unwrap(); // returns a list of directories that have a Cargo.toml file in them and the time it took to find them
//! ```
//!
//! ```rust,no_run
//! use cargo_cleanall::clean_all;
//!
//! clean_all("path/to/directory", false, true).unwrap(); // cleans all project builds without dry run and prints details about the cleaning process
//! ```
//!
//! ## Benchmarks
//!
//! ```bash
//! Timer precision: 100 ns
//! bench                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ clean_all                         │               │               │               │         │
//! │  ├─ 5                222.7 ms      │ 538.4 ms      │ 461.6 ms      │ 464.5 ms      │ 100     │ 100
//! │  ╰─ 10               294.5 ms      │ 788.2 ms      │ 615.5 ms      │ 626.7 ms      │ 100     │ 100
//! ╰─ get_cargo_projects                │               │               │               │         │
//!    ├─ 10               177.9 µs      │ 270 µs        │ 208 µs        │ 207.9 µs      │ 100     │ 100
//!    ├─ 100              590.6 µs      │ 1.025 ms      │ 651.7 µs      │ 663.2 µs      │ 100     │ 100
//!    ╰─ 200              1.024 ms      │ 1.586 ms      │ 1.102 ms      │ 1.117 ms      │ 100     │ 100
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use anyhow::Result;
use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::{
    fs::{read_dir, DirEntry},
    path::{absolute, Path},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

/// Cleans all project builds (`target` directory) by running `cargo clean`
/// in all directories that have a `Cargo.toml` file in them.
///
/// **Note:** Only directories that have a `Cargo.toml` file and a `target` directory will be cleaned.
///
/// **Note:** The search is not recursive and will only clean the top level directories in the path.
///
/// ## Arguments
///
/// * `path` - The path to start searching from
/// * `dry_run` - If true it will only print the commands that would be run
/// * `verbose` - If true it will print details about the cleaning process
///
/// ## Errors
///
/// * [`std::io::Error`] - If there is an error reading the directory
///
/// ## Example
///
/// ```rust,no_run
/// use cargo_cleanall::clean_all;
///
/// clean_all("path/to/directory", false, false).unwrap();
/// ```
pub fn clean_all<P>(path: P, dry_run: bool, verbose: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let (mut dirs, elapsed_get) = get_cargo_projects(path)?;

    dirs.retain(|dir| dir.path().join("target").is_dir());

    if dirs.is_empty() {
        if verbose {
            println!(
                "No Cargo projects found in {}",
                path.display().to_string().green().bold()
            );
        }
        return Ok(());
    }

    if verbose {
        println!(
            "{} Cargo projects found in {}\n",
            dirs.len().to_string().green().bold(),
            format!("{elapsed_get:.2?}").green().bold()
        );
    }

    let count = AtomicUsize::new(0);
    let start = Instant::now();
    dirs.par_iter().for_each(|dir| {
        let path = &dir.path();

        if dry_run {
            count.fetch_add(1, Ordering::SeqCst);
            if verbose {
                println!(
                    "{} {}",
                    "Would clean:".yellow().bold(),
                    absolute(path).unwrap_or_default().display()
                );
            }
            return;
        }

        let output = Command::new("cargo")
            .arg("clean")
            .current_dir(path)
            .output();

        if output.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            count.fetch_add(1, Ordering::SeqCst);
            if verbose {
                println!(
                    "{} {}",
                    "Cleaned:".green().bold(),
                    absolute(path).unwrap_or_default().display()
                );
            }
        } else if verbose {
            let error_msg = output
                .err()
                .map_or("command failed".into(), |e| e.to_string());
            println!(
                "{} {} ({})",
                "Failed to clean:".red().bold(),
                absolute(path).unwrap_or_default().display(),
                error_msg.red()
            );
        }
    });

    let elapsed = start.elapsed();
    if verbose {
        println!(
            "\n{}/{} Cargo projects cleaned in {}",
            count.load(Ordering::SeqCst).to_string().green().bold(),
            dirs.len().to_string().green().bold(),
            format!("{elapsed:.2?}").green().bold()
        );
    }
    Ok(())
}

/// Returns a list of directories that have a Cargo.toml file in them
///
/// ## Arguments
///
/// * `path` - The path to start searching from
///
/// ## Returns
///
/// * `Vec<DirEntry>` - A list of directories that have a Cargo.toml file in them
/// * `Duration` - The time it took to find the directories
///
/// ## Errors
///
/// * [`std::io::Error`] - If there is an error reading the directory
///
/// ## Example
///
/// ```rust,no_run
/// use cargo_cleanall::get_cargo_projects;
///
/// let (dirs, elapsed) = get_cargo_projects("path/to/directory").unwrap();
/// ```
pub fn get_cargo_projects<P>(path: P) -> Result<(Vec<DirEntry>, Duration)>
where
    P: AsRef<Path>,
{
    let start = Instant::now();
    let dirs: Vec<DirEntry> = read_dir(path)?
        .par_bridge()
        .filter_map(std::result::Result::ok)
        .filter(|d| d.file_type().is_ok_and(|ft| ft.is_dir()))
        .filter(|d| d.path().join("Cargo.toml").is_file())
        .collect();
    let elapsed = start.elapsed();

    Ok((dirs, elapsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::{
        fs::{create_dir_all, File},
        io::Write,
    };
    use tempfile::tempdir;

    #[test]
    fn test_clean_all() -> Result<()> {
        setup_test(15, |temp_path, _| {
            clean_all(temp_path, false, false)?;

            for dir in temp_path.read_dir()?.filter_map(std::result::Result::ok) {
                assert!(
                    !dir.path().join("target").is_dir(),
                    "`{}` should not have a target directory",
                    dir.path().display()
                );
            }
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_get_cargo_projects() -> Result<()> {
        setup_test(15, |temp_path, n| {
            let (dirs, _) = get_cargo_projects(temp_path)?;
            assert_eq!(dirs.len(), n);
            Ok(())
        })?;

        Ok(())
    }

    fn setup_test<F>(n: usize, f: F) -> Result<()>
    where
        F: Fn(&Path, usize) -> Result<()>,
    {
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        (0..n)
            .par_bridge()
            .map(|i| {
                let dir = &temp_path.join(format!("dir{i}"));
                create_dir_all(dir)?;

                let mut file = File::create(dir.join("Cargo.toml"))?;
                file.write_all(
                    b"[package]\nname = \"test-proj\"\nversion = \"0.1.0\"\nedition = \"2021\"",
                )?;
                drop(file);

                let src_dir = &dir.join("src");
                create_dir_all(src_dir)?;

                let mut src_file = File::create(src_dir.join("main.rs"))?;
                src_file.write_all(b"fn main() {}")?;
                drop(src_file);

                let build_output = Command::new("cargo")
                    .arg("build")
                    .current_dir(dir)
                    .output()?;

                if !build_output.status.success() {
                    eprintln!(
                        "Failed to build test project in {}: {build_output:?}",
                        dir.display(),
                    );
                    return Err(anyhow!("Cargo build failed in test setup"));
                }

                if !dir.join("target").is_dir() {
                    return Err(anyhow!(
                        "`{}` should have a target directory after build",
                        dir.display()
                    ));
                }

                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;

        f(temp_path, n)
    }
}
