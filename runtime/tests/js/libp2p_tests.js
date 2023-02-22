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
