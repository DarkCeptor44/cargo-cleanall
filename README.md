# cargo-cleanall

Clean all project builds by running `cargo clean` in all directories that have a `Cargo.toml` file in them. The search is not recursive and will only clean the top level directories in the path.

## Getting Started

The `cargo-cleanall` CLI can be installed with the following command:

```bash
# from GitHub
cargo install --git https://github.com/DarkCeptor44/cargo-cleanall

# locally
git clone https://github.com/DarkCeptor44/cargo-cleanall.git
cd cargo-cleanall
cargo install --path .
```

The library can be added to your project with the `cargo add cargo-cleanall` command or by adding the following to your `Cargo.toml`:

```toml
[dependencies]
cargo-cleanall = "^1"
```

## CLI Usage

```bash
$ cargo cleanall -h
Cleans all project builds

Usage: cargo-cleanall.exe cleanall [OPTIONS] [PATH]

Arguments:
  [PATH]  Path where the Rust projects are [default: .]

Options:
      --dry-run  Dry run
  -q, --quiet    Quiet
  -h, --help     Print help
```

## Library Usage

```rust
use cargo_cleanall::{clean_all, get_cargo_projects};

let (dirs, elapsed) = get_cargo_projects("path/to/directory").unwrap(); // returns a list of directories that have a Cargo.toml file in them and the time it took to find them
```

```rust
use cargo_cleanall::clean_all;

clean_all("path/to/directory", false, true).unwrap(); // cleans all project builds without dry run and prints details about the cleaning process
```

## Tests

```bash
cargo test
```

## Benchmarks

You can run the benchmarks with `cargo bench`.

```bash
Timer precision: 100 ns
bench                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ clean_all                         │               │               │               │         │
│  ├─ 5                222.7 ms      │ 538.4 ms      │ 461.6 ms      │ 464.5 ms      │ 100     │ 100
│  ╰─ 10               294.5 ms      │ 788.2 ms      │ 615.5 ms      │ 626.7 ms      │ 100     │ 100
╰─ get_cargo_projects                │               │               │               │         │
   ├─ 10               177.9 µs      │ 270 µs        │ 208 µs        │ 207.9 µs      │ 100     │ 100
   ├─ 100              590.6 µs      │ 1.025 ms      │ 651.7 µs      │ 663.2 µs      │ 100     │ 100
   ╰─ 200              1.024 ms      │ 1.586 ms      │ 1.102 ms      │ 1.117 ms      │ 100     │ 100
```

## License

This project is licensed under the terms of the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0).
