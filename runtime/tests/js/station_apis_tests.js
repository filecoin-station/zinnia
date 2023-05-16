import { assertStrictEquals } from "./vendored/asserts.bundle.js";
import { test } from "zinnia:test";

test("Zinnia.walletAddress", () => {
  // Runtime JS tests are executed with the default configuration
  // In this test, we assert that we can access the wallet address
  // and the value is the default testnet one.
  assertStrictEquals(Zinnia.walletAddress, "t1abjxfbp274xpdqcpuaykwkfb43omjotacm2p3za");
});
