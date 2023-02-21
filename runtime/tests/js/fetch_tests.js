import { assert, assertEquals } from "https://deno.land/std@0.177.0/testing/asserts.ts";

const res = await fetch("https://google.com/");
assertEquals(res.status, 200);
const text = await res.text();
assert(text);
