# Initial Architecture

> Status: ACCEPTED

<!--
PROPOSED, ACCEPTED, REJECTED, DEPRECATED, SUPERSEDED BY {link-to-ADR}
-->

## Context

<!--
What is the issue that we're seeing that motivates this decision or change?
-->

Our grand vision is to enable Filecoin Station to run untrusted modules on
consumer-grade computers operated by non-technical people. This is conceptually
similar to how browsers allow websites to execute untrusted code.

The modules should have access to computer resources - CPU, memory, storage, and
network. However, we want to enforce limits on resource usage, to ensure the
Station is unobtrusive and does not interfere with the actual work performed on
the computer.

We want our runtime to provide high-level APIs for module authors, exposing
primitives for peer-to-peer networking via libp2p, content fetching and
publishing via IPFS & Filecoin, and so on.

We want the ability to share the same underlying libp2p and IPFS node by
multiple Station modules, to avoid the overhead of duplicating connections to
the same network peers.

Modules should be capable of handling high concurrency, leveraging modern design
patterns like asynchronous execution and non-blocking I/O.

## Options Considered

<!--
What are the different options we considered? What are their pros & cons?
-->

### Virtualization (Docker)

Virtualization offers several great benefits:

1. Sandboxing
2. Resource usage limit
3. Modules can be written in any language, composed from multiple heterogeneous
   components (e.g. Nginx reverse proxy + Node.js app)

However, we ruled out virtualization-based solutions for the following reasons:

1. Entry-level computers may not support CPU instructions required for
   virtualization.
2. Often, virtualization must be enabled in BIOS settings.
3. Windows Home edition does not support Microsoft's virtualization framework
   Hyper-V.
4. Sharing the same IPFS & libp2p node between multiple container instances is
   difficult.

### Deno

_https://deno.land_

Deno offers JS & WASM runtime with filesystem and networking APIs, including
capabilities-based sandboxing.

Benefits:

- Full support for both JavaScript/TypeScript and WebAssembly
- Filesystem and network access is sandboxed out of the box
- The engine and platform APIs are designed for asynchrony and non-blocking I/O
- WebAPI-compliant Fetch API implementation
- WebAPIs can be easily called from Rust via
  [wasm-bindgen](https://crates.io/crates/wasm-bindgen) and
  [web-sys](https://crates.io/crates/web-sys) crates.

Issues:

- Missing documentation for embedders

- Static JS imports are not sandboxed. Add this line to your TS script
  `import * as google from 'https://google.com/'` - it will execute even if
  network access is not allowed.

- It's not clear how to implement resource limiting

- It's a VC-funded project with goals that may not align with our goals.

- It provides additional APIs that we may not want to expose to modules. As a
  result, we are likely to spend extra effort to disable or sandbox these APIs.

- User-facing APIs like streams are modelled for TypeScript. Exposing Rust
  crates for Rust/WASM consumption will require unnecessary overhead, switching
  from Rust to TypeScript idioms and then back to Rust again.

### Wasmtime

_https://wasmtime.dev/_

Wasmtime is a fast and secure runtime for WebAssembly, it's a
[Bytecode Alliance](https://bytecodealliance.org) project.

Benefits:

- A community-driven project not depending on any single company

- It implements future proposals to WebAssembly and helps drive innovation in
  the WebAssembly space, e.g. WebAssembly Interface Types and Component Model.

- The runtime is not opinionated about networking & filesystem APIs. It's
  possible to add WASI for filesystem access, but it's also easy to use
  proprietary host APIs instead.

- We have much more control over the I/O layer. It should be easier to implement
  resource usage limiting.

- It implements gas metering & limiting.

- FVM is built on top of Wasmtime too. This can create synergies between our
  teams.

- It supports async host functions.

- Comprehensive documentation for embedders.

Issues:

- Apple M1 architecture has Tier 3 support only (not production ready).

- It's not clear whether functions exported by WASM modules can be async. This
  may require custom glue code to be written.

- There is no stable API for HTTP/HTTPS. WASI does not support opening new
  network connections yet, and API for crypto primitives required by TLS is not
  specified yet.

- Supports WASM only, it cannot run JavaScript/TypeScript code.

### WasmEdge

_https://wasmedge.org/_

WasmEdge aims to bring the cloud-native and serverless application paradigms to
Edge Computing. It's in many aspects similar to Wasmtime. Unfortunately, it's a
C/C++ project, which makes it difficult to integrate into Rust projects,
especially on Apple Silicon, where binaries must be signed.

### Wasmer

_https://wasmer.io_

Wasmer is similar to Wasmtime.

Benefits:

- WebAssembly package manager offering a catalogue of WASM modules

- Supports WASI (filesystem)

- Popular in Web3 space (ChainSafe, Fluence, Hyperledger, and more)

Issues:

- No support for an asynchronous programming model & non-blocking I/O

- Lack of documentation for advanced users. For example, I could not figure out
  how to configure an allow-list of directories allowed to access from WASI.

- Instead of contributing to existing bindgen initiatives, they started a new
  project [fp-bindgen](https://github.com/fiberplane/fp-bindgen) that works with
  Wasmer only.

- The project is owned and run by a single company - Wasmer, Inc.

## Decision

<!--
What is the change that we're proposing and/or doing?
-->

**I am proposing to use Wasmtime as the underlying engine for the Station
Runtime.**

While Deno would allow us to start faster, by providing Fetch API and Filesystem
access with built-in sandboxing, I feel it would make it much harder for us to
build past the built-in features (add IPFS & libp2p, implement resource usage
limiting). Our requirements are unlikely to align with Denoland's vision for
Deno, so we may have a hard time getting our changes accepted.

We are going to build our Runtime incrementally. Early Module authors will
likely need to contribute new features to our Runtime to enable their modules to
be built. This will be easier if the interface between modules and the runtime
is Rust-native. If we built on top of Deno with wasm-bindgen, js-sys and
web-sys, then module authors would need to understand both Rust and JavaScript.

That's why I believe it's better to choose a more bare-bone WASM engine.

From the different WASM engine options out there, it seems that Wasmtime is best
aligned with ProtocolLab's vision of driving technical breakthroughs in the
open. The project is community-driven and contributes to other WASM initiatives.

On the technical side, it is the only engine that supports the asynchronous
programming model (AFAICT). It also has the most extensive documentation.

## Consequences

<!--
What becomes easier or more challenging to do because of this change?
-->

With Wasmtime and our custom high-level networking & filesystem APIs, it should
be easier for us to implement resource usage limits.

By building on top of community-driven initiatives and open standards, we can
contribute our improvements to a wider audience.

With the initial focus on Rust only, we should get faster iteration speed and
make it easier for first module authors to contribute to the runtime too.

Downsides:

- There will be less free stuff at the beginning, we will have to implement
  filesystem access, networking and sandboxing on our own. (But we can
  prioritize different parts and postpone implementation of some of them until
  later.)

- Adding support for JavaScript/TypeScript modules will require additional work
  on implementing a JS-compatible interoperability layer between a JS engine
  like v8 and our WASM host APIs.

## Links &amp; References

<!--
Link to other ADRs, GitHub issues, documentation, etc.
-->

- [Station Runtime Research](https://www.notion.so/pl-strflt/2023-01-Station-Runtime-Research-c45c61a9397241bba98f0d67bafe4e5d)
  in Notion

- [Proof of concepts for different runtimes](https://github.com/filecoin-station/runtime-poc)
