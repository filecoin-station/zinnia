import {
  assert,
  assertEquals,
} from "https://deno.land/std@0.177.0/testing/asserts.ts";

await test("get peer id", () => {
  const id = Zinnia.peerId;
  assertEquals(typeof id, "string");
  assert(
    id.match(/^[0-9a-zA-Z]{52}$/),
    `Expected a PeerId string containing exactly 52 alpha-numeric characters. ` +
      `Actual value has ${id.length} chars: ${id} `,
  );
});

await test("requestProtocol validates remoteAddress", async () => {
  return Zinnia.requestProtocol(123, "/proto", new Uint8Array()).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(
        err.toString(),
        "TypeError: remoteAddress must be string (found: number)",
      );
    },
  );
});

await test("requestProtocol rejects remoteAddress that's not a valid multiaddr with a peer id", async () => {
  return Zinnia.requestProtocol(
    "/ip4/127.0.0.1",
    "/proto",
    new Uint8Array(),
  ).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(
        err.toString(),
        "Error: remote address must contain a valid peer ID",
      );
    },
  );
});

await test("requestProtocol validates protocolName", async () => {
  return Zinnia.requestProtocol("/ipv4", 123, new Uint8Array()).then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(
        err.toString(),
        "TypeError: protocolName must be string (found: number)",
      );
    },
  );
});

await test("requestProtocol validates requestPayload", async () => {
  return Zinnia.requestProtocol("/ipv4", "/proto", "some request payload").then(
    (_) => {
      throw new Error("Zinnia.requestProtocol() should have failed");
    },
    (err) => {
      assertEquals(
        err.toString(),
        "TypeError: requestPayload must be Uint8Array (found: String)",
      );
    },
  );
});

await test("ping remote peer", async () => {
  const request = new Uint8Array(32);
  // FIXME: use Web Crypto to generate random bytes
  // crypto.getRandomValues(request);
  request.set(get32RandomBytes());

  const response = await Zinnia.requestProtocol(
    // FIXME: use a locally running peer instead!
    // The peer below is running https://github.com/bajtos/saturn-interop-libp2p
    "/dns/saturn-link-poc.fly.dev/tcp/3030/p2p/12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk",
    "/ipfs/ping/1.0.0",
    request,
  );

  assert(
    typeof response[Symbol.asyncIterator] === "function",
    "response is an async iterator",
  );

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

async function get32RandomBytes() {
  // The code below creates an array of 32 bytes with the last 4 items based on the current time
  const prefix = [
    165, 48, 99, 103, 1, 164, 242, 58, 43, 138, 224, 125, 245, 150, 27, 208,
    232, 198, 174, 177, 155, 136, 182, 8, 149, 194, 117, 11,
  ];
  const now = Date.now();
  const timeBased = [
    now % 0xff,
    (now >> 8) % 0xff,
    (now >> 16) % 0xff,
    (now >> 24) % 0xff,
  ];
  return [...prefix, ...timeBased];
}

// A dummy wrapper to create isolated scopes for individual tests
// We should eventually replace this with a proper test runner
// See https://github.com/filecoin-station/zinnia/issues/30
async function test(name, fn) {
  try {
    return await fn();
  } catch (err) {
    err.message = `Test ${name} failed. ` + err.message;
    throw err;
  }
}
