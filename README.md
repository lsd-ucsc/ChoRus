<p align="center">
  <img src="./assets/ChoRus.png" width="256" height="256">
</p>

<h1 align="center">ChoRus</h1>

<p align="center"><b>Choreographic Programming in Rust</b></p>

This supplementary material contains the source code of the ChoRus library, examples, and benchmarks.

## Setup

You need to install [Rust](https://www.rust-lang.org/tools/install) to compile the library and examples. The code is tested with Rust 1.73.0.

Alternatively, you can use [Docker](https://www.docker.com/). The following command will create a ephemeral container with Rust 1.73.0 and mount the current directory as `/usr/src/chorus`.

```bash
docker run -it --rm -v "$PWD":/usr/src/chorus -w /usr/src/chorus rust:1.73.0-slim-bullseye bash
```

## `chorus_book`

`chorus_book` contains the documentation of the ChoRus library. You can read it by opening `chorus_book/book/index.html` in your browser.

## `chorus_lib`

`chorus_lib` contains the source code of the ChoRus library, examples, and micro-benchmarks.

### Library Source Code

The source code of the library is located in `chorus_lib/src`. The `core` module contains the core definitions of the library and the `transport` module contains the transport implementations.

### Examples

`chorus_lib/examples` contains examples that illustrate how to use the library. Please refer to the [chorus_lib/examples/README.md](./chorus_lib/examples/README.md) for more information.

In particular, the distributed tic-tac-toe game discussed in Section 6.2 of the paper can be found in `chorus_lib/examples/tic-tac-toe.rs`. The README under `chorus_lib/examples` contains instructions on how to run the game with the HTTP transport.

### Micro-benchmarks

`chorus_lib/benches` contains micro-benchmarks that measure the performance of the library that are discussed in Section 6.3.1. We use [Criterion.rs](https://github.com/bheisler/criterion.rs) for benchmarking. You can run the benchmarks with the following command:

```bash
cd chorus_lib
cargo bench
```

After running the benchmarks, you can see the interactive report in `target/criterion/report/index.html`.

## `kvs`

### Source Code

`kvs` contains the source code of the key-value store example from Section 6.1 and its benchmarks from Section 6.3.2.

- `kvs/src/choreographic.rs` contains the choreographic implementation of the key-value store.
  - `kvs/src/bin/choreographic.rs` is the entry point of the choreographic implementation.
- `kvs/src/handwritten.rs` contains the handwritten implementation of the key-value store.
  - `kvs/src/bin/handwritten.rs` is the entry point of the handwritten implementation.

The two versions of the key-value store can be run with the following commands:

```bash
cargo run --bin <choreographic|handwritten> -- <client|primary|backup>
```

Using the `--bin` flag, you can select the version of the key-value store. After `--`, you can select the role of the process. The client will read a request from the standard input and send it to the server. The request should be in one of the following formats:

- `GET <key>`: Get the value of the key.
- `PUT <key> <value>`: Set the value of the key.

The client, primary, and backup processes will use the HTTP transport and will listen on ports 9000, 9001, and 9002, respectively. Make sure that these ports are not used by other processes.

### Benchmark

The benchmark discussed in Section 6.3.2 can be run with the following command:

```bash
cd kvs
cargo bench
```

After running the benchmarks, you can see the interactive report in `target/criterion/report/index.html`.
