import { test } from "./helpers.js";
import { assert, assertEquals } from "./vendored/asserts.bundle.js";

await test("fetch", async () => {
  const res = await fetch("https://google.com/");
  assertEquals(res.status, 200);
  const text = await res.text();
  assertEquals(typeof text, "string");
  assert(text.includes("<body"));
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

await test("Request", async () => {
  const request = new Request("https://example.com/");
  await request.arrayBuffer();
});

await test("Response", async () => {
  const response = new Response();
  await response.arrayBuffer();
});
