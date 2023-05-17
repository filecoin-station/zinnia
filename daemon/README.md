<h1 align="center">
	<br>
	 🌼
	<br>
	<br>
	Zinnia Daemon
	<br>
	<br>
	<br>
</h1>

[![crates](https://img.shields.io/crates/v/zinnia.svg)](https://crates.io/crates/zinnia)

Zinnia is a runtime for Filecoin Station modules. This crate provides a daemon to run Zinnia Modules
inside Filecoin Station.

## Installation

You can download the `zinniad` binary from
[our GitHub Releases](https://github.com/filecoin-station/zinnia/releases/latest).

| OS      | Platform      | Filename                                                                                                                     |
| ------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Windows | Intel, 64bit  | [zinniad-windows-x64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinniad-windows-x64.zip)       |
| macOS   | Intel, 64bit  | [zinniad-macos-x64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinniad-macos-x64.zip)           |
| macOS   | Apple Silicon | [zinniad-macos-arm64.zip](https://github.com/filecoin-station/zinnia/releases/latest/download/zinniad-macos-arm64.zip)       |
| Linux   | Intel, 64bit  | [zinniad-linux-x64.tar.gz](https://github.com/filecoin-station/zinnia/releases/latest/download/zinniad-linux-x64.tar.gz)     |
| Linux   | ARM, 64bit    | [zinniad-linux-arm64.tar.gz](https://github.com/filecoin-station/zinnia/releases/latest/download/zinniad-linux-arm64.tar.gz) |

### Build from source

If you have Rust tooling installed on your machine (see
[Install Rust](https://www.rust-lang.org/tools/install)), you can build & install Zinnia from the
source code.

```sh
$ cargo install zinniad
```

## Basic use

### Run a JavaScript module

```
FIL_WALLET_ADDRESS=f1... \
zinniad my-module/main.js
```

See [Building Modules](./docs/building-modules.md) for how to write new modules for Filecoin
Station.

> Note: We don't support running more than one Zinnia module in the Filecoin Station yet. Tracking
> issue: [zinnia#144](https://github.com/filecoin-station/zinnia/issues/144)

### Run a Rust module

We have decided to put Rust/WASM modules on hold for now.
