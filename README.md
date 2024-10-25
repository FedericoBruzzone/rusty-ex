## What is what

- [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument): rustc instrumented with a plugin to extract the AST of a Rust program _(full repository, with examples, the important part is `rustc-instrument/rustc-instrument` crate)_

- `rustc_ex`: uses `rustc-instrument` to visit the AST of a Rust program, _from an example in [rustc-instrument](https://github.com/FedericoBruzzone/rustc-instrument)_

- `example-code`: example code to test the `rustc_ex` crate

## Usage

### Test

- `cd rustc_ex`

- `cargo test -- --test-threads=1 --nocapture`

### Cli (`cargo` wrapper)

- `cd rustc_ex/tests/workspaces/first`

- `RUST_LOG=debug cargo run --manifest-path ../../../Cargo.toml --bin cargo-rustc-ex 2>/dev/null`

### Driver (`rustc` wrapper)

- `cd rustc_ex`

- `CARGO_PRIMARY_PACKAGE=1 cargo run --bin rustc-ex-driver -- --cfg 'feature="test"' ../example-code/src/main.rs 2>/dev/null` (specify all features) (without the environment variable, the AST will not be printed)

Optionally:

- `cd rustc_ex/tests/workspaces/first`

- `CARGO_PRIMARY_PACKAGE=1 cargo run --manifest-path ../../../Cargo.toml --bin rustc-ex-driver -- ./src/main.rs`
