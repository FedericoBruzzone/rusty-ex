## What is this?

A `cargo` plugin to analyze and extract a dependency graph (between `cfg` features) from a Rust program.

> [!NOTE]
> `rusty_ex` uses [`rustc-instrument`](https://github.com/FedericoBruzzone/rustc-instrument) to extract and analyze the AST of the Rust program.

## Usage

### Setup

Setup the nightly toolchain:

```bash
rustup toolchain install nightly-2024-12-01
rustup component add --toolchain nightly-2024-12-01 rust-src rustc-dev llvm-tools-preview rust-analyzer clippy
```

### Install the cargo plugin

Install all the binaries:

```bash
cd rusty_ex
cargo install --bins --path .
```

Use the installed binaries:

```bash
cargo-rusty-ex [--PLUGIN_ARG]
deserializer-merger [--PLUGIN_ARG]
rusty-ex-driver [--PLUGIN_ARG]
```

### Test

Run tests on all example workspaces:

```bash
cd rusty_ex
cargo test --no-fail-fast -- --test-threads=1
```

> [!WARNING]
> Some tests _currently_ fail. Run with `--no-fail-fast` to always run all test (even if some early test fails).

### CLI (`cargo` wrapper): `cargo-rusty-ex`

Available plugin args:

- Graphs in DOT format:
  - `--print-terms-tree`: print the terms tree (unified intermediate representation - UIR), including all the terms
  - `--print-features-graph`: print the features graph, including only the dependencies between the features. The weights are based on the nature of the features combinations (`all`, `any`, `not`). Between two nodes there can be at most 1 edge
  - `--print-features-multigraph`: print the features multigraph, including only the dependencies between the features. The weights are based on the nature of the features combinations (`all`, `any`, `not`). Between two nodes there can be multiple edges
  - `--print-artifacts-tree`: print the artifacts tree, including only the AST nodes annotated with a feature. The weights are the size of the artifact (number of child nodes of the node)
- Other:
  - `--print-crate`: print the crate AST
  - `--print-centrality`: print some centralities of the features graph
  - `--print-serialized-graphs`: print the extracted graphs serialized
  - `--print-metadata`: print metadata of extracted graphs in JSON format

Use the installed cargo plugin:

```bash
cd [example_crate_name]
cargo-rusty-ex [--PLUGIN_ARG]

# example:
cd crate_name
cargo-rusty-ex --print-features-graph
```

Use the cargo plugin without installing (from the root of this repository):

```bash
cd rusty_ex/tests/workspaces/[example_crate_name]
cargo run --manifest-path ../../../Cargo.toml --bin cargo-rusty-ex [--CARGO_ARG] -- [--PLUGIN_ARG]

# example:
cd rusty_ex/tests/workspaces/simple_feature_no_weights
cargo run --manifest-path ../../../Cargo.toml --bin cargo-rusty-ex -- --print-features-graph
```

> [!NOTE]
> Additional logs can be enabled by setting the `RUST_LOG` environment variable to `debug`.

> [!NOTE]
> The compilation of the example crates is going to fail. `error: could not compile [example_crate_name]` is expected.

### Run on multiple crates (and merge result): `deserializer-merger`

Serialize the graphs of the crates you want to analyze and save the results in a file:

```bash
cd ~/crate_1
cargo-rusty-ex --print-features-graph > crate_1.json

cd ~/crate_2
cargo-rusty-ex --print-features-graph > crate_2.json
```

Execute the `deserializer-merger`, passing as `-f` argument the files containing the serialization and a plugin arg:
- `--print-terms-tree`: print the terms tree, including all the AST nodes (both annotated with a feature and not)
- `--print-features-graph`: print the features graph, including only the dependencies between the features. The weights are based on the nature of the features combinations (`all`, `any`, `not`)
- `--print-artifacts-tree`: print the artifacts tree, including only the AST nodes annotated with a feature. The weights are the size of the artifact (number of child nodes of the node)

```bash
deserializer-merger [--PLUGIN-ARG] -f crate_1.json -f crate_2.json

# example:
deserializer-merger --print-features-graph -f crate_1.json -f crate_2.json
```
