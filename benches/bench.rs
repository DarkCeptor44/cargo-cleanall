use anyhow::Result;
use cargo_cleanall::{clean_dir, get_cargo_projects};
use divan::{Bencher, black_box};
use std::fs::create_dir_all;
use tempfile::{TempDir, tempdir};
use tokio::runtime::Builder;

const ARGS: &[usize] = &[10, 100];

fn main() {
    divan::main();
}

#[divan::bench(name = "clean_dir (cargo clean)")]
fn bench_clean_dir_cargo(b: Bencher) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    b.with_inputs(|| {
        let temp_dir = setup_test(1).unwrap();
        let dirs = rt.block_on(get_cargo_projects(temp_dir.path())).unwrap();

        (temp_dir, dirs[0].clone())
    })
    .bench_local_refs(|(_t, dir)| black_box(rt.block_on(clean_dir(dir, false, false)).unwrap()));
}

#[divan::bench(name = "clean_dir (fast)")]
fn bench_clean_dir_fast(b: Bencher) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    b.with_inputs(|| {
        let temp_dir = setup_test(1).unwrap();
        let dirs = rt.block_on(get_cargo_projects(temp_dir.path())).unwrap();

        (temp_dir, dirs[0].clone())
    })
    .bench_local_refs(|(_t, dir)| black_box(rt.block_on(clean_dir(dir, true, false)).unwrap()));
}

#[divan::bench(name = "get_cargo_projects", args = ARGS)]
fn bench_get_cargo_projects(b: Bencher, n: usize) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    b.with_inputs(|| setup_test(n).unwrap())
        .bench_local_refs(|dir| black_box(rt.block_on(get_cargo_projects(dir.path())).unwrap()));
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
