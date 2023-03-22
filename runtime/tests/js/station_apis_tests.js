import { assertStrictEquals } from "https://deno.land/std@0.177.0/testing/asserts.ts";

test("Zinnia.walletAddress", () => {
  // Runtime JS tests are executed with the default configuration
  // In this test, we assert that we can access the wallet address
  // and the value is the default testnet one.
  assertStrictEquals(Zinnia.walletAddress, "t1abjxfbp274xpdqcpuaykwkfb43omjotacm2p3za");
});

// A dummy wrapper to create isolated scopes for individual tests
// We should eventually replace this with a proper test runner
// See https://github.com/filecoin-station/zinnia/issues/30
function test(name, fn) {
  try {
    fn();
  } catch (err) {
    err.message = `Test ${name} failed. ` + err.message;
    throw err;
  }
}
