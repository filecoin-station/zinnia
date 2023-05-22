import { test } from "zinnia:test";

test("smoke test", () => {
  // always pass
});

test("failing test", () => {
  throw new Error("this failed");
});
