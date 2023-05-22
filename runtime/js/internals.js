// Capture and export APIs provided by the global Deno object
// This allows internal zinnia modules to access Deno APIs
// even after our bootstrapper deleted the global `Deno` object

export const DenoCore = globalThis.Deno.core;

export function format_test_error(error) {
  return typeof error === "object" && error instanceof Error
    ? DenoCore.ops.op_format_test_error(DenoCore.destructureError(error))
    : error;
}
