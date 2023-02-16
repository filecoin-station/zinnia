# Building modules

A Station Module is a long-running process that's performing jobs like network
probes, content delivery, and computation.

Zinnia provides a JavaScript runtime with a set of platform APIs allowing
modules to interact with the outside world.

In the long run, we want Zinnia to be aligned with the Web APIs as much as
feasible.

For the shorter term, we are going to take shortcuts to deliver a useful
platform quickly.

## Getting started

If you haven't done so, then install `zinnia` CLI per
[our instructions](../cli/README.md#installation).

Using your favourite text editor, create a file called `module.js` with the
following content:

```js
console.log("Hello universe!");
```

Open the terminal and run the module by using `zinnia run` command:

```
$ zinnia run module.js
Hello universe!
```

## Platform APIs

### Standard JavaScript APIs

Zinnia provides all standard JavaScript APIs, you can find the full list in
[MDN web docs](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects).

### Web APIs

The following entities are defined in the global scope (`globalThis`).

#### Console Standard

Zinnia implements most of the `console` Web APIs like `console.log`. You can
find the full list of supported methods in
[Deno docs](https://deno.land/api@v1.30.3?s=Console) and more details about
individual methods in
[MDN web docs](https://developer.mozilla.org/en-US/docs/Web/API/console)

- [console](https://developer.mozilla.org/en-US/docs/Web/API/console)

#### DOM Standard

- [AbortController](https://developer.mozilla.org/en-US/docs/Web/API/AbortController)
- [AbortSignal](https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal)
- [CustomEvent](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent)
- [Event](https://developer.mozilla.org/en-US/docs/Web/API/Event)
- [EventTarget](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget)

#### Encoding Standard

- [TextDecoder](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder)
- [TextEncoder](https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder)
- [TextDecoderStream](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoderStream)
- [TextEncoderStream](https://developer.mozilla.org/en-US/docs/Web/API/TextEncoderStream)

#### HTML Standard

- [ErrorEvent](https://developer.mozilla.org/en-US/docs/Web/API/ErrorEvent)
- [MessageChannel](https://developer.mozilla.org/en-US/docs/Web/API/MessageChannel)
- [MessageEvent](https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent)
- [MessagePort](https://developer.mozilla.org/en-US/docs/Web/API/MessagePort)
- [PromiseRejectionEvent](https://developer.mozilla.org/en-US/docs/Web/API/PromiseRejectionEvent)
- [atob](https://developer.mozilla.org/en-US/docs/Web/API/atob)
- [btoa](https://developer.mozilla.org/en-US/docs/Web/API/btoa)
- [clearInterval](https://developer.mozilla.org/en-US/docs/Web/API/clearInterval)
- [clearTimeout](https://developer.mozilla.org/en-US/docs/Web/API/clearTimeout)
- [reportError](https://developer.mozilla.org/en-US/docs/Web/API/reportError)
- [setInterval](https://developer.mozilla.org/en-US/docs/Web/API/setInterval)
- [setTimeout](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout)
- [structuredClone](https://developer.mozilla.org/en-US/docs/Web/API/structuredClone)

#### Performance & User Timing

- [Performance](https://developer.mozilla.org/en-US/docs/Web/API/Performance)
- [PerformanceEntry](https://developer.mozilla.org/en-US/docs/Web/API/PerformanceEntry)
- [PerformanceMark](https://developer.mozilla.org/en-US/docs/Web/API/PerformanceMark)
- [PerformanceMeasure](https://developer.mozilla.org/en-US/docs/Web/API/PerformanceMeasure)
- [performance](https://developer.mozilla.org/en-US/docs/Web/API/performance)

#### Streams Standard

- [ByteLengthQueuingStrategy](https://developer.mozilla.org/en-US/docs/Web/API/ByteLengthQueuingStrategy)
- [CompressionStream](https://developer.mozilla.org/en-US/docs/Web/API/CompressionStream)
- [CountQueuingStrategy](https://developer.mozilla.org/en-US/docs/Web/API/CountQueuingStrategy)
- [DecompressionStream](https://developer.mozilla.org/en-US/docs/Web/API/DecompressionStream)
- [ReadableByteStreamController](https://developer.mozilla.org/en-US/docs/Web/API/ReadableByteStreamController)
- [ReadableStreamBYOBReader](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBReader)
- [ReadableStreamBYOBRequest](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBRequest)
- [ReadableStreamDefaultController](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultController)
- [ReadableStreamDefaultReader](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader)
- [ReadableStream](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)
- [TransformStreamDefaultController](https://developer.mozilla.org/en-US/docs/Web/API/TransformStreamDefaultController)
- [TransformStream](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream)
- [WritableStreamDefaultController](https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultController)
- [WritableStreamDefaultWriter](https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultWriter)
- [WritableStream](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream)

#### URL Standard

- [URL](https://developer.mozilla.org/en-US/docs/Web/API/URL)
- [URLSearchParams](https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams)
- [URLPattern](https://developer.mozilla.org/en-US/docs/Web/API/URLPattern)

#### WebSockets Standard (partial support)

- [CloseEvent](https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent)

#### Web IDL Standard

- [DOMException](https://developer.mozilla.org/en-US/docs/Web/API/DOMException)

### libp2p

Zinnia comes with a built-in libp2p node based on
[rust-libp2p](https://github.com/libp2p/rust-libp2p). The node is shared by all
Station Modules running on Zinnia. This way we can keep the number of open
connections lower and avoid duplicate entries in routing tables.

The initial version comes with a limited subset of features. We will be adding
more features based on feedback from our users (Station Module builders).

#### Networking stack

- Transport: `tcp` using system DNS resolver
- Multistream-select V1
- Authentication: `noise` with `XX` handshake pattern using X25519 DH keys
- Stream multiplexing: both `yamux` and `mplex`

#### `Zinnia.dialProtocol(multiaddr, protoName)`

Dial a remote peer identified by the `multiaddr` and open a new substream for
the protocol identified by `protoName`. The peer address must include both the
network address and peer id.

```js
const stream = Zinnia.dialProtocol(
  // peer address + peer id
  "/dns/example.com/tcp/3030/p2p/12D3KooWRH7asdJasdr136asXdasd723MasdasdX3qiasda8sasd",
  // protocol name
  "/ipfs/ping/1.0.0",
);
```

**TBD: How should the Stream API look like?**

Ideally, we want to use Web Streams:

- [ReadableStream](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)
- [WritableStream](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream)

```js
// Stream implements both Readable and Writable stream APIs
// (Is that possible with Web Streams? MDN does not describe any DupleStream.)
// The closest example is WebTransportDatagramDuplexStream
// https://developer.mozilla.org/en-US/docs/Web/API/WebTransportDatagramDuplexStream
// Which provides `readable` and `writable` properties to access the read/write stream.
const stream = await node.dialProtocol(targetPeer, "/ipfs/ping/1.0.0");
// write the request
const writableStream = stream.writable;
const writer = writableStream.getWriter();
await writer.write(Buffer.from("some string"));
await writer.close();

// read back the response in chunks
const readableStream = stream.readable;
for await (const chunk of readableStream) {
  const byteArray = chunk;
  // process the chunk
}
```

However, it would be great to be compatible with js-libp2p too.

```js
const stream = await node.dialProtocol(targetPeer, "/ipfs/ping/1.0.0");
// write the request
await stream.sink([Buffer.from("some string")]);
// read back the response in chunks
for await (const chunk of stream.source) {
  const byteArray = chunk.subarray();
  // process the chunk
}
```

In either case, we need to implement the follow Deno ops:

- **dial protocol**: this should return something like Deno duplex stream
  resource identified by resource id (RID)
- **write bytes to RID**: returns number of bytes written?
- **close the writable side of RID**
- **read bytes from RID**: must indicate EOF

<!--
UNSUPPORTED APIs
-->

## Unsupported Web APIs

The following Web APIs are not supported yet.

#### Fetch Standard

Tracking issue: https://github.com/filecoin-station/zinnia/issues/25

- [FormData](https://developer.mozilla.org/en-US/docs/Web/API/FormData)
- [Headers](https://developer.mozilla.org/en-US/docs/Web/API/Headers)
- [ProgressEvent](https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent)
- [Request](https://developer.mozilla.org/en-US/docs/Web/API/Request)
- [Response](https://developer.mozilla.org/en-US/docs/Web/API/Response)
- [fetch](https://developer.mozilla.org/en-US/docs/Web/API/fetch)

#### File API

Tracking issue: n/a

- [Blob](https://developer.mozilla.org/en-US/docs/Web/API/blob)
- [File](https://developer.mozilla.org/en-US/docs/Web/API/File)
- [FileReader](https://developer.mozilla.org/en-US/docs/Web/API/FileReader)

#### Service Workers & Web Workers

Tracking issue: n/a

- [CacheStorage](https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage)
- [Cache](https://developer.mozilla.org/en-US/docs/Web/API/Cache)
- [Worker](https://developer.mozilla.org/en-US/docs/Web/API/Worker)
- [caches](https://developer.mozilla.org/en-US/docs/Web/API/caches)

#### Web Cryptography API

Tracking issue: https://github.com/filecoin-station/zinnia/issues/33

- [CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey)
- [Crypto](https://developer.mozilla.org/en-US/docs/Web/API/Crypto)
- [SubtleCrypto](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto)
- [crypto](https://developer.mozilla.org/en-US/docs/Web/API/crypto)

#### WebSockets Standard

Tracking issue: n/a

- [WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)

#### Other

- `XMLHttpRequest` Standard
