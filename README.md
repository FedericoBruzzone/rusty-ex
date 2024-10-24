## What is what

- [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument): rustc instrumented with a plugin to extract the AST of a Rust program _(full repository, with examples, the important part is `rustc-instrument/rustc-instrument` crate)_

- `ast-visitor`: uses `rustc-instrument` to visit the AST of a Rust program, _from an example in [rustc-instrument](https://github.com/FedericoBruzzone/rustc-instrument)_

- `example-code`: example code to test the `ast-visitor` crate

## Usage

### Driver

- `cd ast-visitor`
- `CARGO_PRIMARY_PACKAGE=1 cargo run --bin rustc-ex-driver -- --cfg 'feature="ciao"' ../example-code/src/main.rs 2>/dev/null` (specify all features) (without the environment variable, the AST will not be printed)

