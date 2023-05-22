// Capture and export APIs provided by the global Deno object
// This allows internal zinnia modules to access Deno APIs
// even after our bootstrapper deleted the global `Deno` object

export const DenoCore = globalThis.Deno.core;
