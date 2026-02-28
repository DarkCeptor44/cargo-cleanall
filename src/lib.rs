//! # cargo-cleanall
//!
//! Provides a way to get a list of directories in a path that have a `Cargo.toml` file in them. The search is not recursive and will only clean the top level directories in the path.
//!
//! ## Getting Started
//!
//! Run `cargo add --git https://github.com/DarkCeptor44/cargo-cleanall` or add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargo-cleanall = { version = "2.0.0", git = "https://github.com/DarkCeptor44/cargo-cleanall" }
//! ```
//!
//! ## Library Usage
//!
//! ```rust,ignore
//! use cargo_cleanall::get_cargo_projects;
//!
//! // returns a list of directories that have a Cargo.toml file in them
//! let dirs = get_cargo_projects("path/to/directory").await.unwrap();
//! ```
//!
//! ```rust,ignore
//! use cargo_cleanall::clean_dir;
//!
//! // cleans the project build with a cargo command
//! let cleaned = clean_dir("path/to/directory", false, false).await.unwrap();
//!
//! // cleans the project build by directly removing the target directory, this is way faster
//! let cleaned = clean_dir("path/to/directory", true, false).await.unwrap();
//! ```
//!
//! ## Benchmarks
//!
//! ```bash
//! Timer precision: 100 ns
//! bench                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ clean_dir (cargo clean)  125.9 ms      │ 141.5 ms      │ 131.7 ms      │ 132.1 ms      │ 100     │ 100
//! ├─ clean_dir (fast)         275.1 µs      │ 3.503 ms      │ 346.1 µs      │ 397.8 µs      │ 100     │ 100
//! ╰─ get_cargo_projects                     │               │               │               │         │
//!    ├─ 10                    246.3 µs      │ 673.5 µs      │ 324.1 µs      │ 352.7 µs      │ 100     │ 100
//!    ╰─ 100                   765.8 µs      │ 3.788 ms      │ 946.4 µs      │ 1.071 ms      │ 100     │ 100
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use anyhow::{Context, Result, anyhow};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{metadata, remove_dir_all},
    process::Command,
    task::JoinSet,
};

/// Cleans a project build (`target` directory) by running `cargo clean` or manually removing the `target` directory.
///
/// **Note:** Only directories that have a `Cargo.toml` file and a `target` directory will be cleaned.
///
/// ## Arguments
///
/// * `path` - The path of the directory to clean
/// * `fast` - If `true` it will remove the `target` directory directly instead of spawning a `cargo clean` command
/// * `dry_run` - If `true` it will exit immediately after checking if `path` and `target` exist
///
/// ## Errors
///
/// Returns an error if there is an error reading the directory, if there is an error reading the file, if there is an error running the `cargo clean` command, or if there is an error removing the `target` directory
///
/// ## Examples
///
/// ```rust,ignore
/// use cargo_cleanall::clean_dir;
///
/// let cleaned = clean_dir("path/to/directory", false, false).await.unwrap();
/// ```
pub async fn clean_dir<P>(path: P, fast: bool, dry_run: bool) -> Result<bool>
where
    P: AsRef<Path>,
{
    clean_dir_impl(path.as_ref(), fast, dry_run).await
}

async fn clean_dir_impl(path: &Path, fast: bool, dry_run: bool) -> Result<bool> {
    let dir_meta = metadata(path)
        .await
        .context("failed to get metadata for path")?;

    if !dir_meta.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", path.display()));
    }

    let target_path = path.join("target");
    let target_meta = match metadata(&target_path).await {
        Ok(m) => m,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).context("failed to get metadata for target"),
    };

    if !target_meta.is_dir() {
        return Ok(false);
    }

    if dry_run {
        return Ok(true);
    }

    if fast {
        remove_dir_all(&target_path)
            .await
            .context("failed to remove target directory")?;
        return Ok(true);
    }

    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(path)
        .output()
        .await
        .context("failed to build Command")?;

    if output.status.success() {
        Ok(true)
    } else {
        Err(anyhow!("failed to clean: {}", path.display()))
    }
}

/// Returns a list of directories that have a `Cargo.toml` file in them
///
/// **Note:** The search is not recursive and will only return the top level directories in the path.
///
/// ## Arguments
///
/// * `path` - The path to start searching from
///
/// ## Returns
///
/// * `Result<Vec<PathBuf>>` - A list of directories that have a `Cargo.toml` file
///
/// ## Errors
///
/// Returns an error if there is an error reading the directory, or if there is an error reading the file
///
/// ## Examples
///
/// ```rust,ignore
/// use cargo_cleanall::get_cargo_projects;
///
/// let dirs = get_cargo_projects("path/to/directory").await.unwrap();
/// ```
pub async fn get_cargo_projects<P>(path: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    get_cargo_projects_impl(path.as_ref()).await
}

async fn get_cargo_projects_impl(path: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = tokio::fs::read_dir(path).await?;
    let mut set = JoinSet::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let entry_path = entry.path();
        set.spawn(async move {
            if let Ok(ft) = entry.file_type().await {
                if ft.is_dir() {
                    if let Ok(meta) = metadata(entry_path.join("Cargo.toml")).await {
                        if meta.is_file() {
                            return Some(entry_path);
                        }
                    }
                }
            }
            None
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(Some(p)) = res {
            results.push(p);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::{TempDir, tempdir};

    #[tokio::test]
    async fn test_clean_dir_cargo() -> Result<()> {
        let temp_dir = setup_test(15)?;
        let dirs = get_cargo_projects(temp_dir.path()).await?;

        for dir in dirs {
            assert!(clean_dir(&dir, false, false).await?);
            assert!(
                !dir.join("target").is_dir(),
                "`{}` should not have a target directory",
                dir.display()
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_dir_fast() -> Result<()> {
        let temp_dir = setup_test(15)?;
        let dirs = get_cargo_projects(temp_dir.path()).await?;

        for dir in dirs {
            assert!(clean_dir(&dir, true, false).await?);
            assert!(
                !dir.join("target").is_dir(),
                "`{}` should not have a target directory",
                dir.display()
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cargo_projects() -> Result<()> {
        let n = 15;
        let temp_dir = setup_test(n)?;
        let dirs = get_cargo_projects(temp_dir.path()).await?;

        assert_eq!(dirs.len(), n);
        for i in 0..n {
            assert!(dirs.contains(&temp_dir.path().join(format!("dir{i}"))));
        }
        Ok(())
    }

    // TODO consider if its worth making this non-blocking/async
    fn setup_test(n: usize) -> Result<TempDir> {
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        for i in 0..n {
            let dir = temp_path.join(format!("dir{i}"));
            create_dir_all(&dir)?;

            std::fs::write(
                dir.join("Cargo.toml"),
                b"[package]\nname = \"test-proj\"\nversion = \"0.1.0\"\nedition = \"2024\"",
            )?;

            let src_dir = dir.join("src");
            create_dir_all(&src_dir)?;

            std::fs::write(src_dir.join("main.rs"), b"fn main() {}")?;

            create_dir_all(dir.join("target"))?;
        }

        Ok(temp_dir)
    }
}
