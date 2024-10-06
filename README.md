## What is what

- [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument): rustc instrumented with a plugin to extract the AST of a Rust program _(full repository, with examples, the important part is `rustc-instrument/rustc-instrument` crate)_

- `ast-visitor`: uses `rustc-instrument` to visit the AST of a Rust program, _from an example in [rustc-instrument](https://github.com/FedericoBruzzone/rustc-instrument)_

- `example-code`: example code to test the `ast-visitor` crate

## Usage

- `cd ast-visitor`
- `CARGO_PRIMARY_PACKAGE=1 cargo run ../example-code/main.rs` (without the environment variable, the AST will not be printed)
