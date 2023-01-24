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

  <a href="https://www.nasa.gov/image-feature/first-flower-grown-in-space-stations-veggie-facility">
    <em>Zinnia was the first ever flower grown in space.</em>
  </a>
</div>

## Architecture

![](./docs/images/runtime-diagram.png)

### Components

- **JS/WASM engine:** [wasmtime](https://wasmtime.dev), see the
  [Initial Architecture decision record](docs/architecture-decision-records/2023-01-initial-architecture.md)
- **Non-blocking I/O:** _TBD_
- **Networking**: _TBD_
- **IPFS:** _TBD_
- **Block (K/V) storage:** _TBD_
