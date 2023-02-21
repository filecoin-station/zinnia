import { assert, assertEquals } from "https://deno.land/std@0.177.0/testing/asserts.ts";

test("AbortController", () => {
  assertEquals(typeof AbortController, "function", "typeof AbortController");
  assertEquals(AbortController.name, "AbortController", "AbortController.name");
});

test("atob & btoa", () => {
  assertEquals(btoa("some text"), "c29tZSB0ZXh0", `btoa("some text)`);
  assertEquals(atob("c29tZSB0ZXh0"), "some text", `atob("c29tZSB0ZXh0")`);
});

await test("fetch", async () => {
  const res = await fetch("https://google.com/");
  assertEquals(res.status, 200);
  const text = await res.text();
  assert(text);
});

await test("FormData", async () => {
  const formData = new FormData();
  formData.append("name", "value");
});

await test("Headers", async () => {
  const headers = new Headers();
  headers.append("name", "value");
});

await test("ProgressEvent", async () => {
  const event = new ProgressEvent();
  assertEquals(event.total, 0);
});

await test("TextEncoder", () => {
  const encoder = new TextEncoder();
  const bytes = encoder.encode("€");
  assertEquals(Array.from(bytes.values()), [226, 130, 172]);
});

await test("TextDecoder", () => {
  let decoder = new TextDecoder();
  let bytes = new Uint8Array([226, 130, 172]);
  let text = decoder.decode(bytes);
  assertEquals(text, "€");
});

await test("URL", () => {
  const url = new URL("https://filstation.app");
  assertEquals(url.host, "filstation.app");
});

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
