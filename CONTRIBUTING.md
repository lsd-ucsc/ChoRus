## Developing ChoRus

### Prerequisites

You will need the following tools to develop ChoRus:

- [Rust](https://www.rust-lang.org/tools/install)
  - We recommend using [rustup](https://rustup.rs/) to install Rust.
- [mdBook](https://rust-lang.github.io/mdBook/)
  - `cargo install mdbook`

### Repository Structure

This repository is set up as a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). Currently, there are two crates in the workspace:

- `chorus_lib` is a library that contains the core functionality of ChoRus. Users can install this crate as a library and use it in their own projects.
- `chorus_derive` is an internal crate that contains the procedural macros used by `chorus_lib`. It is a dependency of `chorus_lib` and not intended to be used directly by users.

### Writing Examples

The `examples` directory under `chorus_lib` contains several examples of how to use ChoRus. To run an example, use the following command:

```bash
cargo run --example $(EXAMPLE_NAME)
```

It is recommended to write examples for new features.

### Testing

Many of the ChoRus files contain unit tests. Some examples also contain integration tests. You can use `cargo` to run all tests:

```bash
cargo test
```

The documentation also contains tests. Please refer to the [documentation](#documentation) section for more information.

### Documentation

ChoRus uses [mdBook](https://rust-lang.github.io/mdBook/) to generate documentation and all features must be properly documented. Documentation sources are located in the `chorus_book` directory. You can use the following command to preview the documentation:

```bash
mdbook serve chorus_book --open
```

Most of the code snippets in the documentation are tested using `mdbook test`. You can use the following command at the repository root to run tests in the documentation:

```bash
cargo clean
cargo build
mdbook test chorus_book -L ./target/debug/deps
```

Cleaning the project ensures that `mdbook` can find the `chorus_lib` library. More information can be found [on this mdbook issue](https://github.com/rust-lang/mdBook/issues/706).
