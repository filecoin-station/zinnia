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

You can download the `zinnia` binary from
[our GitHub Releases](https://github.com/filecoin-station/zinnia/releases/latest).

| OS      | Platform      | Filename                                                                                                               |
| ------- | ------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Windows | Intel, 64bit  | [zinnia-windows-x64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinnia-windows-x64.zip)   |
| macOS   | Intel, 64bit  | [zinnia-macos-x64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinnia-macos-x64.zip)       |
| macOS   | Apple Silicon | [zinnia-macos-arm64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinnia-macos-arm64.zip)   |
| Linux   | Intel, 64bit  | [zinnia-linux-x64.tar.gz](https://github.com/filecoin-station/zinnia/releases/latest/download/zinnia-linux-x64.tar.gz) |

### Build from source

If you have Rust tooling installed on your machine (see
[Install Rust](https://www.rust-lang.org/tools/install)), you can build & install Zinnia from the
source code.

You also need the Protocol Buffers compiler, `protoc`. See
[Protocol Buffer Compiler Installation](https://grpc.io/docs/protoc-installation/).

Run the following command to build and install Zinnia:

```sh
$ cargo install zinnia
```

## Basic use

### Run a JavaScript module

```
zinnia run my-module.js
```

See [Building Modules](./docs/building-modules.md) for how to write new modules for Filecoin
Station.

### Run a Rust module

We have decided to put Rust/WASM modules on hold for now.
