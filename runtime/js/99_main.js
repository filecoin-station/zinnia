// ZINNIA VERSION: Copyright 2023 Protocol Labs. All rights reserved. MIT OR Apache-2.0 license.
// ORIGINAL WORK: Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// https://github.com/denoland/deno/blob/34bfa2cb2c1f0f74a94ced8fc164e81cc91cb9f4/runtime/js/99_main.js

"use strict";

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
    ObjectSetPrototypeOf,
  } = window.__bootstrap.primordials;
  const eventTarget = window.__bootstrap.eventTarget;
  const timers = window.__bootstrap.timers;
  const colors = window.__bootstrap.colors;
  const inspectArgs = window.__bootstrap.console.inspectArgs;
  const quoteString = window.__bootstrap.console.quoteString;
  const libp2p = window.__bootstrap.libp2p;

  const { windowOrWorkerGlobalScope, mainRuntimeGlobalProperties } =
    window.__bootstrap.globalScope;

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
    core.setMacrotaskCallback(timers.handleTimerMacrotask);
    // core.setMacrotaskCallback(promiseRejectMacrotaskCallback);
    // core.setWasmStreamingCallback(fetch.handleWasmStreaming);
    // core.setReportExceptionCallback(reportException);
    ops.op_set_format_exception_callback(formatException);
    // version.setVersions(
    //   runtimeOptions.denoVersion,
    //   runtimeOptions.v8Version,
    //   runtimeOptions.tsVersion
    // );
    // build.setBuildInfo(runtimeOptions.target);
    // util.setLogDebug(runtimeOptions.debugFlag, source);
    colors.setNoColor(runtimeOptions.noColor || !runtimeOptions.isTty);
    // deno-lint-ignore prefer-primordials
    Error.prepareStackTrace = core.prepareStackTrace;
    // registerErrors();
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
    ObjectDefineProperties(globalThis, mainRuntimeGlobalProperties);
    ObjectSetPrototypeOf(globalThis, Window.prototype);

    if (runtimeOptions.inspectFlag) {
      const consoleFromDeno = globalThis.console;
      wrapConsole(consoleFromDeno, consoleFromV8);
    }

    eventTarget.setEventTargetData(globalThis);

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
