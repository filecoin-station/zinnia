import { test } from "zinnia:test";
import { assertEquals, assertNotEquals } from "zinnia:assert";

test("getRandomValues()", async () => {
  const first = new Uint8Array(4);
  crypto.getRandomValues(first);
  const second = new Uint8Array(4);
  crypto.getRandomValues(second);

  assertNotEquals(first, second);
});

test("generateKey(), sign() and verify()", async () => {
  const keyPair = await crypto.subtle.generateKey(
    {
      name: "ECDSA",
      namedCurve: "P-384",
    },
    true,
    ["sign", "verify"],
  );

  const message = "Hello world!";
  const payload = new TextEncoder().encode(message);

  const algo = { name: "ECDSA", hash: { name: "SHA-384" } };
  const signature = await crypto.subtle.sign(algo, keyPair.privateKey, payload);
  assertEquals(signature.byteLength, 96);

  const result = (await crypto.subtle.verify(algo, keyPair.publicKey, signature, payload))
    ? "signature verified"
    : "invalid signature";

  assertEquals(result, "signature verified");
});
