import { test } from "zinnia:test";
import { assertMatch, assertEquals, assertArrayIncludes } from "zinnia:assert";

console.log(Zinnia.versions);

test("Zinnia.versions", () => {
  assertArrayIncludes(Object.keys(Zinnia), ["versions"], "Zinnia properties");
  assertEquals(typeof Zinnia.versions, "object", "typeof Zinnia.versions");
});

test("Zinnia.versions.zinnia", () => {
  assertArrayIncludes(Object.keys(Zinnia.versions), ["zinnia"], "Zinnia.versions properties");
  assertMatch(Zinnia.versions.zinnia, /^\d+\.\d+\.\d+$/);
});

test("Zinnia.versions.v8", () => {
  assertArrayIncludes(Object.keys(Zinnia.versions), ["v8"], "Zinnia.versions properties");
  assertMatch(Zinnia.versions.v8, /^\d+\.\d+\.\d+\.\d+$/);
});
