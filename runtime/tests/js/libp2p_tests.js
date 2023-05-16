import { assert, assertEquals } from "./vendored/asserts.bundle.js";
import { test } from "zinnia:test";

test("get peer id", () => {
  const id = Zinnia.peerId;
  assertEquals(typeof id, "string");
  assert(
    id.match(/^[0-9a-zA-Z]{52}$/),
    `Expected a PeerId string containing exactly 52 alpha-numeric characters. ` +
      `Actual value has ${id.length} chars: ${id} `,
  );
});

test("requestProtocol validates remoteAddress", async () => {
  return Zinnia.requestProtocol(123, "/proto", new Uint8Array()).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(err.toString(), "TypeError: remoteAddress must be string (found: number)");
    },
  );
});

test("requestProtocol rejects remoteAddress that's not a valid multiaddr with a peer id", async () => {
  return Zinnia.requestProtocol("/ip4/127.0.0.1", "/proto", new Uint8Array()).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(err.toString(), "Error: remote address must contain a valid peer ID");
    },
  );
});

test("requestProtocol validates protocolName", async () => {
  return Zinnia.requestProtocol("/ipv4", 123, new Uint8Array()).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(err.toString(), "TypeError: protocolName must be string (found: number)");
    },
  );
});

test("requestProtocol validates requestPayload", async () => {
  return Zinnia.requestProtocol("/ipv4", "/proto", "some request payload").then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(err.toString(), "TypeError: requestPayload must be Uint8Array (found: String)");
    },
  );
});

test("ping remote peer", async () => {
  const request = new Uint8Array(32);
  crypto.getRandomValues(request);

  const response = await Zinnia.requestProtocol(
    // FIXME: use a locally running peer instead!
    // The peer below is running https://github.com/bajtos/saturn-interop-libp2p
    "/dns/saturn-link-poc.fly.dev/tcp/3030/p2p/12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk",
    "/ipfs/ping/1.0.0",
    request,
  );

  assert(typeof response[Symbol.asyncIterator] === "function", "response is an async iterator");

  const chunks = [];
  for await (const c of response) {
    chunks.push(c);
  }

  // The response should have been read in a single chunk
  // and should be the same as the request payload
  assertEquals(chunks, [request]);

  // The chunk should be Uint8Array
  assertEquals(chunks[0].constructor, Uint8Array);
});
