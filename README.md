# cargo-cleanall

Clean all project builds by either running `cargo clean` or directly removing the `target` directory in all directories that have a `Cargo.toml` file in them. The search is not recursive and will only clean the top level directories in the path.

## MSRV

| Version | MSRV | Edition |
|:-------:|:----:| :-----: |
| 2.x.y   | 1.85 |  2024   |
| 1.x.y   | 1.80 |  2021   |

## Getting Started

The `cargo-cleanall` CLI can be installed with the following command:

```bash
# from GitHub
cargo install --git https://github.com/DarkCeptor44/cargo-cleanall --features cli

# locally
git clone https://github.com/DarkCeptor44/cargo-cleanall.git
cd cargo-cleanall
cargo install --path . --features cli
```

The library can be added to your project with the `cargo add --git https://github.com/DarkCeptor44/cargo-cleanall` command or by adding the following to your `Cargo.toml`:

```toml
[dependencies]
cargo-cleanall = { version = "2.0.0", git = "https://github.com/DarkCeptor44/cargo-cleanall" }
```

## CLI Usage

```bash
$ cargo cleanall -h
Cleans all project builds

Usage: cargo-cleanall.exe cleanall [OPTIONS] [PATH]

Arguments:
  [PATH]  Path where the Rust projects are [default: .]

Options:
  -f, --fast           Fast mode (removes target directory directly, not recommended)
      --dry-run        Dry run
  -l, --limit <LIMIT>  Concurrency limit [default: 64]
  -h, --help           Print help
```

## Library Usage

```rust
use cargo_cleanall::get_cargo_projects;

// returns a list of directories that have a Cargo.toml file in them
let dirs = get_cargo_projects("path/to/directory").await.unwrap();
```

```rust
use cargo_cleanall::clean_dir;

// cleans the project build without dry run and by running a `cargo clean` command
let cleaned = clean_dir("path/to/directory", false, false).await.unwrap();

// cleans the project build without dry run and by directly removing the `target` directory.
// this is way faster but it might not be as safe or effective
let cleaned = clean_dir("path/to/directory", true, false).await.unwrap();
```

## Tests

```bash
cargo test
```

## Benchmarks

You can run the benchmarks with `cargo bench`.

```bash
Timer precision: 100 ns
bench                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ clean_dir (cargo clean)  125.9 ms      │ 141.5 ms      │ 131.7 ms      │ 132.1 ms      │ 100     │ 100
├─ clean_dir (fast)         275.1 µs      │ 3.503 ms      │ 346.1 µs      │ 397.8 µs      │ 100     │ 100
╰─ get_cargo_projects                     │               │               │               │         │
   ├─ 10                    246.3 µs      │ 673.5 µs      │ 324.1 µs      │ 352.7 µs      │ 100     │ 100
   ╰─ 100                   765.8 µs      │ 3.788 ms      │ 946.4 µs      │ 1.071 ms      │ 100     │ 100
```

## License

This project is licensed under the terms of the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0).
