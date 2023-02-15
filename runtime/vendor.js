// Run this Deno script to fetch and store Deno modules
// This is a temporary workaround until we support module imports
// See https://github.com/filecoin-station/zinnia/issues/43
//
// Run this script using the following command:
//   deno --allow-all runtime/vendor.js

import { fromFileUrl } from "https://deno.land/std@0.177.0/path/mod.ts";

await vendor(
  "https://deno.land/std@0.177.0/testing/asserts.ts",
  "asserts.bundle.js",
);

async function vendor(url, outfile) {
  const outpath = fromFileUrl(import.meta.resolve(`./vendored/${outfile}`));
  const cmd = ["deno", "bundle", url, "--", outpath];
  const child = Deno.run({ cmd });
  const status = await child.status();
  child.close();
  if (!status.success) {
    const reason = status.code
      ? `code ${status.code}`
      : `signal ${status.signal}`;

    throw new Error(`Process failed with ${reason}: ${cmd}`);
  }
}
