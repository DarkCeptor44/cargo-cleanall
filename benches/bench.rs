use cargo_cleanall::{clean_all, get_cargo_projects};
use divan::{black_box, Bencher};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::{tempdir, TempDir};

/// Helper struct to manage the temp dirs and proj setup for benches
struct BenchmarkSetup {
    _temp_dir: TempDir,
    temp_path: PathBuf,
    num_projects: usize,
}

impl BenchmarkSetup {
    fn new(n: usize) -> Self {
        let temp_dir = tempdir().expect("Failed to create temp dir for benchmark setup");
        let temp_path = temp_dir.path().to_path_buf();

        (0..n).par_bridge().for_each(|i| {
            let dir = &temp_path.join(format!("dir{i}"));
            create_dir_all(dir).expect("Failed to create project dir");

            let mut file =
                File::create(dir.join("Cargo.toml")).expect("Failed to create Cargo.toml");
            file.write_all(
                b"[package]\nname = \"test-proj\"\nversion = \"0.1.0\"\nedition = \"2021\"",
            )
            .expect("Failed to write Cargo.toml");

            let src_dir = &dir.join("src");
            create_dir_all(src_dir).expect("Failed to create src dir");
            let mut src_file =
                File::create(src_dir.join("main.rs")).expect("Failed to create main.rs");
            src_file
                .write_all(b"fn main() {}")
                .expect("Failed to write main.rs");

            let build_output = Command::new("cargo")
                .arg("build")
                .current_dir(dir)
                .output()
                .expect("Failed to run cargo build in benchmark setup");

            if !build_output.status.success() {
                eprintln!(
                    "Failed to build test project in {}: {build_output:?}",
                    dir.display(),
                );
                panic!(
                    "Cargo build failed in benchmark setup for {}",
                    dir.display()
                );
            }

            if !dir.join("target").is_dir() {
                panic!(
                    "`{}` should have a target directory after build",
                    dir.display()
                );
            }
        });

        Self {
            _temp_dir: temp_dir,
            temp_path,
            num_projects: n,
        }
    }

    fn path(&self) -> &Path {
        &self.temp_path
    }
}

fn main() {
    divan::main();
}

#[divan::bench(name="clean_all", args = [5, 10])]
fn bench_clean_all(b: Bencher, n: usize) {
    let setup = BenchmarkSetup::new(n);
    let path_to_bench = setup.path();
    let num_projects = setup.num_projects;

    b.bench(|| {
        (0..num_projects).par_bridge().for_each(|i| {
            let dir = path_to_bench.join(format!("dir{i}"));
            Command::new("cargo")
                .arg("build")
                .current_dir(&dir)
                .output()
                .expect("Failed to rebuild project for clean_all benchmark");
        });

        clean_all(black_box(path_to_bench), false, false).unwrap();
        black_box(());
    });
}

#[divan::bench(name = "get_cargo_projects", args = [10, 100, 200])]
fn bench_get_cargo_projects(b: Bencher, n: usize) {
    let setup = BenchmarkSetup::new(n);
    let path_to_bench = setup.path();

    b.bench(|| {
        black_box(get_cargo_projects(black_box(path_to_bench)).unwrap());
    });
}
