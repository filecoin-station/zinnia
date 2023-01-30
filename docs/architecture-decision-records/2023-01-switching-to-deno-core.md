# Switching to Deno Core

> Status: PROPOSED

<!--
PROPOSED, ACCEPTED, REJECTED, DEPRECATED, SUPERSEDED BY {link-to-ADR}
-->

## Context

After we decided to build Zinnia on top of `wasmtime` (see
[Initial Architecture](./2023-01-initial-architecture.md)), I started to look
into implementing non-trivial APIs which require data exchange between the host
and the module and quickly hit limitations. Even a simple `log(msg: string)` API
requires non-trivial glue code.

While there is ongoing work to standardize the higher-level interfaces and build
better tooling, it's too early. The WebAssembly Interface Types proposal was
abandoned in favour of WebAssembly Components. The WIT tooling
([wit-bindgen](https://github.com/bytecodealliance/wit-bindgen)) was updated to
support components, it's not considered stable and discourages users from
depending on it.

While we were willing to put some extra effort into building our flavour of
platform APIs like network access, having to build an interoperability layer for
exchanging data between the host and the module feels like a poor use of our
limited bandwidth.

Also, after discussing our decisions with more people, we realized it's more
important to allow JavaScript developers to build Station Modules than we
originally thought.

We also realized that we can easily hide (most of) the underlying architecture
choices from module authors by providing an SDK with a higher-level API of the
Station Runtime. If we ever decide to change the underlying engine, then we
should be able to re-implement our SDKs to target this new engine, and module
authors will just have to rebuild their modules with the new SDK version.

We decided to take another look at v8 and evaluate it as a possible low-level
JS/WASM engine for Zinnia.

## Options Considered

### Raw v8

v8 is a C++ project. Fortunately, the [Deno](https://deno.land) project
maintains a [Rust wrapper](https://crates.io/crates/v8) converting v8's C++ APIs
into APIs closer to idiomatic Rust.

Unfortunately, working with v8 this way is still too cumbersome.

See e.g. their example code here:
[v8 example](https://docs.rs/v8/0.62.0/v8/#example)

### `deno_core`

The Deno project is composed from many modules. One of them is `deno_core`,
which is a library that implements an opinionated way of structuring Rust code
and exposing it for consumption from JavaScript.

It does not provide any platform APIs like networking and filesystem access.

The host provides a set of Ops (operations) implemented as Rust functions.
`deno_core` takes care of binding these Rust functions to v8 runtime, exposing
them to the JavaScript world. The bindings also seamlessly convert between
JavaScript and Rust types, e.g. `number[]` and `Vec<f64>`.

I wrote a small PoC where the host provides two functions (sync
`log(msg: string)` and async `sleep(duration: number)`), runs two modules (one
in JS, one in Rust) and each module invokes both host functions. Source code:
[runtime-poc/deno-core](https://github.com/filecoin-station/runtime-poc/tree/main/deno-core)

### `deno_runtime`

When using `deno_core`, it's up to us to implement all higher-level
functionality. What if we could pull the implementation of these features from
the Deno runtime, instead of implementing them ourselves?

Deno's next building block is `deno_runtime`, which bundles `deno_core` with the
implementation of different ops and builds a high-level JS API on top of that.

Unfortunately, based on a brief investigation, it seems that `deno_runtime` is
designed as a self-contained thing that's not open to be extended.

However, the implementations of ops are packages into crates, which are easy to
add into a `deno_core`-based project.

## Decision

We decided to pivot and build Zinnia on top of `deno_core`.

It will allow us to iterate much faster, while still having a lot of control of
the API exposed to modules.

While Deno is not perfect, we need to prioritize iteration speed at this stage
of the project.

## Consequences

- We can support JS & Rust/WASM module from the beginning.

- We can leverage what the Deno team learned about Rust & v8 integration over
  the years and follow their best practices.

- We can cherry-pick functionality from Deno, either import some of their crates
  or copy parts of their code (as a last resort solution).

### Caveats

- Supporting both JS and Rust/WASM requires more work. Maybe we should pick one
  as the first-class option with great documentation & DX, and keep the second
  one less polished.

- Since Deno is a projected stewarded by a single VC-backed company, there is a
  risk that the project may become less maintained or may change the direction
  in a way that's not compatible with us. Because we are not building directly
  on top of the user-facing `deno` tooling, we have several options how to
  handle that situation. We can fork the libraries we are building on top of, or
  even rebuild Zinnia runtime & SDK using a different technology.

## Links &amp; References

- [The Internals of Deno: How does Deno execute programs?](https://choubey.gitbook.io/internals-of-deno/)

- Proof of concept:
  https://github.com/filecoin-station/runtime-poc/tree/main/deno-core
