# Getting Started

## Installation

ChoRus is still under active development. We are still expecting to make breaking changes to the API. For this reason, we do not publish ChoRus to crates.io just yet. For this reason, you will need to use a git dependency to use ChoRus in your project.

```bash
# create a binary crate
cargo new chorus_hello_world
cd chorus_hello_world
# install ChoRus as a dependency
cargo add --git https://github.com/lsd-ucsc/ChoRus.git chorus_lib
```

## Running Hello World example

Once you have installed ChoRus, you can run the Hello World example by copy-pasting the following code into `main.rs`:

```rust
{{#include ../../chorus_lib/examples/hello.rs}}
```
