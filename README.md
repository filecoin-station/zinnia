<h1 align="center">
	<br>
	 🌼
	<br>
	<br>
	Zinnia
	<br>
	<br>
	<br>
</h1>

Zinnia is a runtime for Filecoin Station modules.

It provides a sandboxed environment to execute untrusted code on consumer-grade
computers.

<div align="center">
  <img src="https://s.yimg.com/uu/api/res/1.2/WtLPXqGgiUashZzP.J4drw--~B/Zmk9ZmlsbDtoPTU4Mzt3PTg3NTthcHBpZD15dGFjaHlvbg--/https://o.aolcdn.com/hss/storage/midas/229be0287167454b558989b2e29221d8/203272974/zinnias-success.jpg.cf.jpg" width="50%" />

  <br>
  <a href="https://www.nasa.gov/image-feature/first-flower-grown-in-space-stations-veggie-facility">
    <em>Zinnia was the first ever flower grown in space.</em>
  </a>
</div>

## Architecture

![](./docs/images/runtime-diagram.png)

### Components

- **JS/WASM engine:** [deno_core](https://crates.io/crates/deno_core), see the
  decision record for
  [Switching to Deno Core](docs/architecture-decision-records/2023-01-switching-to-deno-core.md)
- **Non-blocking I/O:** _TBD_
- **Networking**: _TBD_
- **IPFS:** _TBD_
- **Block (K/V) storage:** _TBD_

## Installation

To install Zinnia, you need to have Rust tooling installed on your machine. See
[Install Rust](https://www.rust-lang.org/tools/install).

Then you can install Zinnia using the `cargo`:

```sh
$ cargo install zinnia
```

This will build Zinnia and all its dependencies from the sources, it can take
while. In the future, we want to simplify the installation process, see
[#23](https://github.com/filecoin-station/zinnia/issues/23).

## Basic use

```sh
$ zinnia run module.js
```

See [Building Modules](./docs/building-modules.md) for how to write new modules
for Filecoin Station.
