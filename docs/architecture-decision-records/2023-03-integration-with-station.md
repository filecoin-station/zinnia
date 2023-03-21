# Zinnia in Filecoin Station

<!--
This is a minimal template. Feel free to add more sections as needed.

Please review also the Design Doc template and add any relevant sections to your ADR:
https://www.notion.so/pl-strflt/Writing-a-Design-Doc-aa6034be43c2434ba88a2fd844516e94
-->

> Status: ACCEPTED

<!--
PROPOSED, ACCEPTED, REJECTED, DEPRECATED, SUPERSEDED BY {link-to-ADR}
-->

## Context

Zinnia is a runtime for Filecoin Station modules. It's time to integrate Zinnia into the Station to
enable the deployment of the first modules.

Zinnia is designed to run multiple modules inside the same process, using V8 Isolates for keeping
modules separated and sandboxed. The integration with libp2p (and later IPFS) is designed to allow
multiple modules to share the same underlying set of network connections and block storage.

Zinnia has two primary modes of operation:

1. A developer tool for building a single module. We want to optimize for ease of use and a fast
   feedback loop.

   In this mode, Zinnia is typically started and stopped frequently, running for short periods at a
   time, executing a single module only.

   As a CLI tool, it reads the configuration from a config file (either user-level or
   project-specific) or CLI arguments. There should be reasonable defaults tailored to developers
   building a module - e.g. the state files should be stored in the project's working directory.

2. A deployment platform, running modules inside Filecoin Station instances operated by
   non-technical users. We want to optimize for a smooth user experience for Station operators
   first, and easy troubleshooting of operations by module developers second.

   In this mode, Zinnia is a long-running process executing multiple modules. It needs to support
   reloading individual modules when a new version of a module is deployed. It also needs to report
   various information back to the Station, e.g. activity log and the number of jobs completed.

   As a service, it should read the configuration from environment variables (see
   [The Twelve-Factor App](https://12factor.net/config)) and allow embedders to improve security
   e.g. by keeping modules in a different place from the runtime state.

### `zinniad`

For the reasons above, I propose to build a new binary called `zinniad` that will execute Zinnia
modules inside the Station.

_(This is a strawman proposal, subject to changes during implementation.)_

Configuration via environment variables:

- `FIL_WALLET_ADDRESS`: Address of Station's built-in wallet (required).

- `STATE_ROOT`: Directory where to keep state files (optional). Defaults to
  `$XDG_STATE_HOME/zinniad`.

Positional arguments:

- Positional arguments specify which modules to run, where each module is a single JS file. We don't
  make any assumptions about the directory layout of modules. Paths are resolved relatively to the
  current working directory.

Example invocation:

```bash
cd /Applications/Filecoin\ Station.app/Contents/Resources/zinnia-modules

FIL_WALLET_ADDRESS=f1etc \
STATE_ROOT=$HOME/Library/Caches/Filecoin\ Station/zinnia \
zinniad \
  saturn-l2/main.js \
  ping.js \
  retrieval-checker/dist/index.js
```

### Communication with Station (Core, Desktop):

As explained above, I propose to use environment variables to pass configuration from the Station to
Zinnia.

For communicating information from Zinnia back to the Station, I propose that Zinnia prints
newline-delimited JSON entries to `stdout` and uses `stderr` for general/debug logging. This format
is easy to parse from Node.js code powering both Station variants.

Example messages:

- **Activity log - error**

  `{"type": "activity:error", "module": "saturn", "message": "Cannot connect to the orchestrator."x }`

  _Note: `"module": "saturn"` describes which module emitted the log._

- **Activity log - info**

  `{"type": "activity:info", "module": null, "message": "Zinnia is starting up..."}`

  _Note: `"module": null` means the message comes from Zinnia runtime._

- **Number of jobs completed:**

  `{"type": "jobs-completed", "total": 123 }`

  _Note: This message is emitted periodically, e.g. every 200ms._

  In the future, we can easily extend this line to include per-module stats too:

  `{"type": "jobs-completed", "total": 123, modules: {"saturn": 100, "retrieval-checker": 23}}`

### Module identifiers

We need each module to have a unique identifier (a name) that we can use in the messages above. This
id must remain unchanged across module version upgrades.

For the initial version, these ids will be hard-coded human-readable names like `saturn-l2`.

In the future, when we move towards untrusted modules deployed in a decentralized manner, we will
need to find a different way how to derive these unique module ids. That's out of scope of the
current work though. Adding new id types should be easy as long as our architecture supports
arbitrary string ids.

### Deploying and upgrading modules

The initial version will not implement any upgrade mechanisms for modules. Both `zinniad` and all
module sources will be bundled inside the Station. When a new module is added or an existing module
is upgraded to a new version, we will publish a new version of the Station.

### Zinnia API for module builders

_(This is a strawman proposal, subject to changes during implementation.)_

```ts
namespace Zinnia {
  // omitted: existing APIs like `peerId`

  /** Get the wallet address, this value is typically provided by the Station. */
  walletAddress: String;

  /** Report activities to the Station */
  log: {
    /** Report an informative status update, e.g. "Connecting to the network." */
    info(message: string);

    /** Report an error, e.g. "Cannot connect to the orchestrator." */
    error(message: string);
  }

  /** Report completion of a single job */
  jobCompleted();
}
```

### Dev-mode in `zinnia`

These APIs will behave differently when running a module via `zinnia` CLI in development.

- In the initial version, the wallet address is hardcoded to a dummy testnet address
  `t1abjxfbp274xpdqcpuaykwkfb43omjotacm2p3za`. This value is taken from
  [Filecoin Lotus docs](https://lotus.filecoin.io/lotus/manage/manage-fil/#public-key-address) with
  the leading `"f"` replaced with `"t"`.

  Later, we can implement reading of the wallet address from a configuration file, e.g.
  `.zinnia/config.yaml` in the current working directory (typically the project root).

- Activity logs are printed to stdout with human-readable formatting.
  ```
  [10:30:20.000 INFO ] Connecting to the network.
  [10:30:21.000 ERROR] Cannot connect to the orchestrator.
  ```
- Job completions are printed to stdout but less frequently, e.g. every 500ms.

  ```
  [10:30:20.000 STATS] Jobs completed: 123
  [10:30:20.500 STATS] Jobs completed: 134
  [10:30:21.000 STATS] Jobs completed: 146
  ```

## Options Considered

1. Don't build a new binary, bundle the existing `zinnia` CLI inside the Station.

   Pros:

   - Less work to ship the first version. Less yak-shaving like setting up CI/CD workflows.

   Cons:

   - Zinnia modules cannot share libp2p & IPFS resources (network connection, peer address book,
     block store).

   - To meet the requirements of both module builders and the Station runtime, we would need to
     implement extra configuration options to get different behaviour in different settings.

   - Most of the Station work will be discarded later, once we need `zinniad` to allow running
     multiple modules inside the same runtime process.

2. Don't push job stats via stdout, let the station pull the stats via HTTP API. (The current
   saturn-l2 module uses this model.)

   Pros:

   - We already have code in Station to deal with this.
   - Less cluttered `stdout`

   Cons:

   - More complex implementation in Zinnia for little benefits. Since `zinniad` routes all
     `console.log` messages to `stderr`, no humans should be reading `stdout`, therefore extra
     clutter does not matter.

   - More complex integration between Station and Zinnia: Zinnia needs to report URL where the stats
     API is available, the Station needs to parse that URL from Zinnia's `stdout`.

3. Include a timestamp in the JSON messages printed to `stdout` for the Station. We decided this is
   not needed now and can be easily added later if such a need arises.

   Our current goal is integration with the Station. The Station (Core or Desktop) and the Zinnia
   runtime will initially sit on the same machine. If the log consumer sits on the same machine, it
   shouldn't matter who will attach the timestamp.

4. Use `XDG_STATE_HOME` to configure where should `zinniad` keep the state files.

   The major difference between `XDG_STATE_HOME` and `STATE_ROOT` is that `XDG_STATE_HOME` provides
   a system (or user) wide directory, we need to append a zinnia-specific segment to that path to
   obtain `STATE_ROOT`.

   I prefer to give the user full control over the location by providing them `STATE_ROOT` config
   option.

   This becomes relevant when Zinnia is running inside the Station. If we use `XDG_STATE_HOME`, then
   we will keep the state in `XDG_STATE_HOME/zinnia`, a different place from where Station keeps its
   files. We could make this path Station specific, but that feels hacky to me and incorrect in the
   situation when Zinnia runs outside of the Station

   However, I think it's a good idea to make `STATE_ROOT` an optional configuration option and use
   `XDG_STATE_HOME/zinnia` as the default value.

<!--
What are the different options we considered? What are their pros & cons?
-->

## Decision

<!--
What is the change that we're proposing and/or doing?
-->

Build `zinniad` and add new `Zinnia` APIs as described above.

## Consequences

<!--
What becomes easier or more challenging to do because of this change?
-->

After we implement the architecture described above and integrate Zinnia into the Station, we will
have a solid foundation that's easily extensible and should not require major updates for a long
time.

<!--
## Links &amp; References

Link to other ADRs, GitHub issues, documentation, etc.
-->
