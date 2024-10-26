## What is what

- [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument): rustc instrumented with a plugin to extract the AST of a Rust program _(full repository, with examples, the important part is `rustc-instrument/rustc-instrument` crate)_

- `rustc_ex`: uses `rustc-instrument` to visit the AST of a Rust program, _from an example in [rustc-instrument](https://github.com/FedericoBruzzone/rustc-instrument)_

## Usage

### Test

- `cd rustc_ex`

- `cargo test -- --test-threads=1 --nocapture`

### Cli (`cargo` wrapper)

- `cd rustc_ex/tests/workspaces/first`

- `RUST_LOG=debug cargo run --manifest-path ../../../Cargo.toml --bin cargo-rustc-ex` (`-- --help`)

Optionally:

- `LD_LIBRARY_PATH=$(rustc --print sysroot)/lib RUST_LOG=debug ../../../target/debug/cargo-rustc-ex` (`--help`)

### Driver (`rustc` wrapper)

*Find a way to pass to the driver the plugin args using "PLUGIN_ARGS" environment variable*

- `cd rustc_ex`

- `CARGO_PRIMARY_PACKAGE=1 RUST_LOG=debug cargo run --bin rustc-ex-driver -- ../example-code/src/main.rs  --cfg 'feature="test"'` (without the environment variable, the driver will not work)

Optionally:

- `cd rustc_ex/tests/workspaces/first`

- `CARGO_PRIMARY_PACKAGE=1 RUST_LOG=debug cargo run --manifest-path ../../../Cargo.toml --bin rustc-ex-driver -- ./src/main.rs`
