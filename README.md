## What is this?

A `cargo` plugin to analyze and extract a dependency graph (between `cfg` features) from a Rust program.

> [!NOTE]
> `rustc_ex` uses [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument) to extract and analyze the AST of the Rust program.

## Usage

### Setup

Setup the nightly toolchain:

```bash
rustup toolchain install nightly-2024-10-18
rustup component add --toolchain nightly-2024-10-18 rust-src rustc-dev llvm-tools-preview rust-analyzer clippy
```

### Test

Run tests on all example workspaces:

```bash
cd rustc_ex
cargo test --no-fail-fast -- --test-threads=1
```

> [!WARNING]
> Some tests _currently_ fail. Run with `--no-fail-fast` to always run all test (even if some early test fails).

### Cli (`cargo` wrapper)

Available plugin args:

- Graphs in DOT format:
  - `--print-ast-graph`: print the AST graph, including all the AST nodes (both annotated with a feature and not)
  - `--print-features-graph`: print the features graph, including only the dependencies between the features. The weights are based on the nature of the features combinations (`all`, `any`, `not`)
  - `--print-artifacts-graph`: print the artifacts graph, including only the AST nodes annotated with a feature. The weights are the size of the artifact (number of child nodes of the node)
- Other:
  - `--print-crate`: print the crate AST
  - `--print-centrality`: print some centralities of the features graph
  - `--print-serialized-graphs`: print the extracted graphs serialized

Use the cargo plugin:

```bash
cd rustc_ex/tests/workspaces/[example_crate_name]
cargo run --manifest-path ../../../Cargo.toml --bin cargo-rustc-ex [--CARGO_ARG] -- [--PLUGIN_ARG]
```

> [!NOTE]
> Additional logs can be enabled by setting the `RUST_LOG` environment variable to `debug`.

> [!TIP]
> Example:
> ```bash
> cd rustc_ex/tests/workspaces/simple_feature_no_weights
> cargo run --manifest-path ../../../Cargo.toml --bin cargo-rustc-ex -- --print-features-graph
> ```

> [!NOTE]
> The compilation of the example crates is going to fail. `error: could not compile [example_crate_name]` is expected.

### Driver (`rustc` wrapper)

> [!CAUTION]
> It is not currently possible to pass the plugin args to the driver without using an environment variable. Using the CLI is advised.

TODO: Find a way to pass to the driver the plugin args using "PLUGIN_ARGS" environment variable

```bash
cd rustc_ex
CARGO_PRIMARY_PACKAGE=1 cargo run --bin rustc-ex-driver -- ./tests/workspaces/simple_feature_no_weights/src/main.rs --cfg 'feature="test"'
```

Or:

```bash
cd rustc_ex/tests/workspaces/simple_feature_no_weights
CARGO_PRIMARY_PACKAGE=1 cargo run --manifest-path ../../../Cargo.toml --bin rustc-ex-driver -- ./src/main.rs
```
