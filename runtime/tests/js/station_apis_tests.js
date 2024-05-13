import { test } from "zinnia:test";
import { assertStrictEquals } from "zinnia:assert";

test("Zinnia.walletAddress", () => {
  // Runtime JS tests are executed with the default configuration
  // In this test, we assert that we can access the wallet address
  // and the value is the default testnet one.
  assertStrictEquals(Zinnia.walletAddress, "0x000000000000000000000000000000000000dEaD");
});

test("Zinnia.stationId", () => {
  assertStrictEquals(Zinnia.stationId, "0".repeat(88));
});
