// ZINNIA VERSION: Copyright 2023 Protocol Labs. All rights reserved. MIT OR Apache-2.0 license.
// ORIGINAL WORK: Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// See https://github.com/denoland/deno/blob/main/runtime/js/98_global_scope.js

"use strict";

((window) => {
  const core = Deno.core;

  const Console = window.__bootstrap.console.Console;

  // from Deno's window.__bootstrap.util
  function nonEnumerable(value) {
    return {
      value,
      writable: true,
      enumerable: false,
      configurable: true,
    };
  }

  // https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope
  const windowOrWorkerGlobalScope = {
    console: nonEnumerable(
      new Console((msg, level) => core.print(msg, level > 1))
    ),
  };

  window.__bootstrap.globalScope = {
    windowOrWorkerGlobalScope,
  };
})(this);
