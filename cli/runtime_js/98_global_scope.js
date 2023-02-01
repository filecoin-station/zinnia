"use strict";

// See https://github.com/denoland/deno/blob/main/runtime/js/98_global_scope.js

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
      new Console((msg, level) => core.print(msg, level > 1)),
    ),
  };

  window.__bootstrap.globalScope = {
    windowOrWorkerGlobalScope,
  };
})(this);
