"use strict";

// See https://github.com/denoland/deno/blob/main/runtime/js/99_main.js

// Removes the `__proto__` for security reasons.
// https://tc39.es/ecma262/#sec-get-object.prototype.__proto__
delete Object.prototype.__proto__;

// Remove Intl.v8BreakIterator because it is a non-standard API.
delete Intl.v8BreakIterator;

((window) => {
  const core = Deno.core;
  const ops = core.ops;
  const {
    Error,
    ErrorPrototype,
    ObjectDefineProperties,
    ObjectPrototypeIsPrototypeOf,
  } = window.__bootstrap.primordials;
  const colors = window.__bootstrap.colors;
  const inspectArgs = window.__bootstrap.console.inspectArgs;
  const quoteString = window.__bootstrap.console.quoteString;
  const { windowOrWorkerGlobalScope } = window.__bootstrap.globalScope;

  function formatException(error) {
    if (ObjectPrototypeIsPrototypeOf(ErrorPrototype, error)) {
      return null;
    } else if (typeof error == "string") {
      return `Uncaught ${inspectArgs([quoteString(error)], {
        colors: !colors.getNoColor(),
      })}`;
    } else {
      return `Uncaught ${inspectArgs([error], {
        colors: !colors.getNoColor(),
      })}`;
    }
  }

  function runtimeStart(runtimeOptions) {
    ops.op_set_format_exception_callback(formatException);
    colors.setNoColor(runtimeOptions.noColor || !runtimeOptions.isTty);
    // deno-lint-ignore prefer-primordials
    Error.prepareStackTrace = core.prepareStackTrace;
  }

  let hasBootstrapped = false;

  function bootstrapMainRuntime(runtimeOptions) {
    if (hasBootstrapped) {
      throw new Error("Worker runtime already bootstrapped");
    }

    core.initializeAsyncOps();

    const consoleFromV8 = window.Deno.core.console;
    const wrapConsole = window.__bootstrap.console.wrapConsole;

    // Remove bootstrapping data from the global scope
    delete globalThis.__bootstrap;
    delete globalThis.bootstrap;
    hasBootstrapped = true;

    ObjectDefineProperties(globalThis, windowOrWorkerGlobalScope);

    if (runtimeOptions.inspectFlag) {
      const consoleFromDeno = globalThis.console;
      wrapConsole(consoleFromDeno, consoleFromV8);
    }

    runtimeStart(runtimeOptions);

    // delete `Deno` global
    delete globalThis.Deno;
  }

  ObjectDefineProperties(globalThis, {
    bootstrap: {
      value: {
        mainRuntime: bootstrapMainRuntime,
      },
      configurable: true,
    },
  });
})(this);
