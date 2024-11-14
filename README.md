<p align="center">
  <img src="./assets/ChoRus.png" width="256" height="256">
</p>

<h1 align="center">ChoRus</h1>

<p align="center"><b>Choreographic Programming in Rust</b></p>

This supplementary material contains the source code of the ChoRus library, examples, and benchmarks.

## Setup

You need to install [Rust](https://www.rust-lang.org/tools/install) to compile the library and examples. The code is tested with Rust 1.81.0.

Alternatively, you can use [Docker](https://www.docker.com/). The following command will create a ephemeral container with Rust 1.81.0 and mount the current directory as `/usr/src/chorus`.

```bash
docker run -it --rm -v "$PWD":/usr/src/chorus -w /usr/src/chorus rust:1.81.0-slim-bullseye bash
```

## `chorus_lib`

`chorus_lib` contains the source code of the ChoRus library, examples, and micro-benchmarks.

### Library Source Code

The source code of the library is located in `chorus_lib/src`. The `core` module contains the core definitions of the library and the `transport` module contains the transport implementations.

The API documentation of the library can be generated with the following command:

```bash
cargo doc --open -p chorus_lib
```

### Examples

`chorus_lib/examples` contains examples that illustrate how to use the library. You can run an example with the following command:

```bash
cargo run --example $(EXAMPLE_NAME)
```
