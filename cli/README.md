<h1 align="center">
	<br>
	 🌼
	<br>
	<br>
	Zinnia CLI
	<br>
	<br>
	<br>
</h1>

[![crates](https://img.shields.io/crates/v/zinnia.svg)](https://crates.io/crates/zinnia)

Zinnia is a runtime for Filecoin Station modules. This crate provides the actual `zinnia`
executable.

## Installation

To install Zinnia, you need to have Rust tooling installed on your machine. See
[Install Rust](https://www.rust-lang.org/tools/install).

You also need the Protocol Buffers compiler, `protoc`. See
[Protocol Buffer Compiler Installation](https://grpc.io/docs/protoc-installation/)

Then you can install Zinnia using `cargo`:

```sh
$ cargo install zinnia
```

This will build Zinnia and all its dependencies from source, which can take a while. In the future,
we want to simplify the installation process, see
[#23](https://github.com/filecoin-station/zinnia/issues/23).

## Basic use

### Run a JavaScript module

```
zinnia run my-module.js
```

See [Building Modules](./docs/building-modules.md) for how to write new modules for Filecoin
Station.

### Run a Rust module

We have decided to put Rust/WASM modules on hold for now.
