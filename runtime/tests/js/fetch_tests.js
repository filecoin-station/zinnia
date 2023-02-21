import { assert, assertEquals } from "https://deno.land/std@0.177.0/testing/asserts.ts";

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