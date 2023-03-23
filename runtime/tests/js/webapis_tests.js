import { assertEquals } from "https://deno.land/std@0.181.0/testing/asserts.ts";

test("AbortController", () => {
  assertEquals(typeof AbortController, "function", "typeof AbortController");
  assertEquals(AbortController.name, "AbortController", "AbortController.name");
});

test("atob & btoa", () => {
  assertEquals(btoa("some text"), "c29tZSB0ZXh0", `btoa("some text)`);
  assertEquals(atob("c29tZSB0ZXh0"), "some text", `atob("c29tZSB0ZXh0")`);
});

test("TextEncoder", () => {
  const encoder = new TextEncoder();
  const bytes = encoder.encode("€");
  assertEquals(Array.from(bytes.values()), [226, 130, 172]);
});

test("TextDecoder", () => {
  let decoder = new TextDecoder();
  let bytes = new Uint8Array([226, 130, 172]);
  let text = decoder.decode(bytes);
  assertEquals(text, "€");
});

test("URL", () => {
  const url = new URL("https://filstation.app");
  assertEquals(url.host, "filstation.app");
});

// A dummy wrapper to create isolated scopes for individual tests
// We should eventually replace this with a proper test runner
// See https://github.com/filecoin-station/zinnia/issues/30
function test(name, fn) {
  try {
    return fn();
  } catch (err) {
    err.message = `Test ${name} failed. ` + err.message;
    throw err;
  }
}
