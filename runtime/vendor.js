// Run this Deno script to fetch and store Deno modules
// This is a temporary workaround until we support module imports
// See https://github.com/filecoin-station/zinnia/issues/43
//
// Run this script using the following command:
//   deno run --allow-run --allow-read --allow-write runtime/vendor.js

const STD_VERSION = "0.226.0";

import { fromFileUrl } from "https://deno.land/std@0.181.0/path/mod.ts";
import { green } from "https://deno.land/std@0.183.0/fmt/colors.ts";

let assertsPath = await vendor(`jsr:@std/assert@0.226.0`, "asserts.bundle.js");
await patchAssertsBundle(assertsPath);
await patchDocs();

async function vendor(url, outfile) {
  const outpath = fromFileUrl(import.meta.resolve(`./js/vendored/${outfile}`));
  const cmd = ["deno", "bundle", url, "--", outpath];
  const child = Deno.run({ cmd });
  const status = await child.status();
  child.close();
  if (!status.success) {
    const reason = status.code ? `code ${status.code}` : `signal ${status.signal}`;

    throw new Error(`Process failed with ${reason}: ${cmd}`);
  }
  return outpath;
}

async function patchAssertsBundle(assertsPath) {
  await patchFile(assertsPath, (content) =>
    content
      .replace(
        'const noColor = typeof Deno?.noColor === "boolean" ? Deno.noColor : false;',
        "const noColor = false;",
      )
      .replaceAll(
        "const { Deno } = globalThis;",
        // Deno.inspect is exposed via Zinnia.inspect
        "const { Zinnia: Deno } = globalThis;",
      ),
  );
}

async function patchDocs() {
  let buildingModules = fromFileUrl(import.meta.resolve("../docs/building-modules.md"));
  return patchFile(buildingModules, (content) =>
    content.replace(/jsr\.io\/@std\/assert@\d+\.\d+\.\d+/g, `jsr.io/@std/assert@${STD_VERSION}`),
  );
}

async function patchFile(filePath, fn) {
  const orig = await Deno.readTextFile(filePath);
  const patched = fn(orig);
  await Deno.writeTextFile(filePath, patched);
  console.log("%s %s", green("Patched"), filePath);
}
