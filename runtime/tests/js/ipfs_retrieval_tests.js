import { test } from "zinnia:test";
import { assertEquals, assertMatch, assertRejects, AssertionError } from "zinnia:assert";

const EXPECTED_CAR_BASE64 =
  "OqJlcm9vdHOB2CpYJQABcBIgO/KicpaH2Kj0sXyJNWLdY4kGpEe2mjY5zovBGRJ+6mpndmVyc2lvbgFrAXASIDvyonKWh9io9LF8iTVi3WOJBqRHtpo2Oc6LwRkSfupqCkUIAhI/TXkgbW9zdCBmYW1vdXMgZHJhd2luZywgYW5kIG9uZSBvZiB0aGUgZmlyc3QgSSBkaWQgZm9yIHRoZSBzaXRlGD8=";

test("can retrieve CID content as a CAR file", async () => {
  const requestUrl = "ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni";
  const response = await fetch(requestUrl);
  await assertResponseIsOk(response);

  const payload = await response.arrayBuffer();
  assertEquals(payload.byteLength, 167, "CAR size in bytes");

  const payload_encoded = btoa(String.fromCharCode(...new Uint8Array(payload)));
  assertEquals(payload_encoded, EXPECTED_CAR_BASE64);

  assertEquals(response.url, requestUrl);
});

test("can retrieve IPFS content using URL", async () => {
  const requestUrl = new URL("ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni");
  const response = await fetch(requestUrl);
  await assertResponseIsOk(response);

  const payload = await response.arrayBuffer();
  assertEquals(payload.byteLength, 167, "CAR size in bytes");

  assertEquals(response.url, requestUrl.toString());
});

test("can retrieve IPFS content using Fetch Request object", async () => {
  const request = new Request("ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni");
  const response = await fetch(request);
  await assertResponseIsOk(response);

  const payload = await response.arrayBuffer();
  assertEquals(payload.byteLength, 167, "CAR size in bytes");

  assertEquals(response.url, request.url);
});

test("does not modify original request headers - headers initialized as array", async () => {
  const request = new Request(
    "ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni",
    { headers: [["X-Test", "true"]] },
  );
  const response = await fetch(request);
  await assertResponseIsOk(response);

  assertEquals(Array.from(request.headers.keys()), ["x-test"]);
});

test("does not modify original request headers - headers initialized as object", async () => {
  const request = new Request(
    "ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni",
    { headers: { "X-Test": "true" } },
  );
  const response = await fetch(request);
  await assertResponseIsOk(response);

  assertEquals(Array.from(request.headers.keys()), ["x-test"]);
});

test("does not modify original request headers - headers initialized as Headers", async () => {
  const request = new Request(
    "ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni",
    { headers: new Headers({ "X-Test": "true" }) },
  );
  const response = await fetch(request);
  await assertResponseIsOk(response);

  assertEquals(Array.from(request.headers.keys()), ["x-test"]);
});

test("rejects user-provided Authorization header", async () => {
  const request = new Request(
    "ipfs://bafybeib36krhffuh3cupjml4re2wfxldredkir5wti3dttulyemre7xkni",
    { headers: { Authorization: "invalid" } },
  );

  let error = await assertRejects(() => fetch(request));
  assertMatch(error.message, /authorization/i);
});

/**
 * @param {Response} response Fetch API response
 */
async function assertResponseIsOk(response) {
  if (!response.ok) {
    throw new AssertionError(
      `Fetch request failed with status code ${response.status}: ${await response.text()}`,
    );
  }
}
