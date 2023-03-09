// ZINNIA VERSION: Copyright 2023 Protocol Labs. All rights reserved. MIT OR Apache-2.0 license.
// ORIGINAL WORK: Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// https://github.com/denoland/deno/blob/86785f21194460d713276dca2/runtime/js/99_main.js

// Removes the `__proto__` for security reasons.
// https://tc39.es/ecma262/#sec-get-object.prototype.__proto__
delete Object.prototype.__proto__;

// Remove Intl.v8BreakIterator because it is a non-standard API.
delete Intl.v8BreakIterator;

const core = globalThis.Deno.core;
const ops = core.ops;
const primordials = globalThis.__bootstrap.primordials;
const {
  DateNow,
  Error,
  ErrorPrototype,
  ObjectDefineProperties,
  ObjectPrototypeIsPrototypeOf,
  ObjectSetPrototypeOf,
} = primordials;
import * as util from "internal:zinnia_runtime/js/06_util.js";
import * as event from "internal:deno_web/02_event.js";
import * as timers from "internal:deno_web/02_timers.js";
import * as colors from "internal:deno_console/01_colors.js";
import { inspectArgs, quoteString, wrapConsole } from "internal:deno_console/02_console.js";
import * as performance from "internal:deno_web/15_performance.js";
import {
  mainRuntimeGlobalProperties,
  windowOrWorkerGlobalScope,
} from "internal:zinnia_runtime/js/98_global_scope.js";

function formatException(error) {
  if (ObjectPrototypeIsPrototypeOf(ErrorPrototype, error)) {
    return null;
  } else if (typeof error == "string") {
    return `Uncaught ${inspectArgs([quoteString(error)], {
      colors: !colors.getNoColor(),
    })}`;
  } else {
    return `Uncaught ${inspectArgs([error], { colors: !colors.getNoColor() })}`;
  }
}

function runtimeStart(runtimeOptions) {
  core.setMacrotaskCallback(timers.handleTimerMacrotask);
  // core.setMacrotaskCallback(promiseRejectMacrotaskCallback);
  // core.setWasmStreamingCallback(fetch.handleWasmStreaming);
  // core.setReportExceptionCallback(event.reportException);
  ops.op_set_format_exception_callback(formatException);
  // version.setVersions(
  //   runtimeOptions.denoVersion,
  //   runtimeOptions.v8Version,
  //   runtimeOptions.tsVersion,
  // );
  // build.setBuildInfo(runtimeOptions.target);
  // util.setLogDebug(runtimeOptions.debugFlag, source);
  colors.setNoColor(runtimeOptions.noColor || !runtimeOptions.isTty);
  // deno-lint-ignore prefer-primordials
  Error.prepareStackTrace = core.prepareStackTrace;
}

let hasBootstrapped = false;

function bootstrapMainRuntime(runtimeOptions) {
  if (hasBootstrapped) {
    throw new Error("Worker runtime already bootstrapped");
  }

  performance.setTimeOrigin(DateNow());

  const consoleFromV8 = globalThis.Deno.core.console;

  // Remove bootstrapping data from the global scope
  delete globalThis.__bootstrap;
  delete globalThis.bootstrap;
  util.log("bootstrapMainRuntime");
  hasBootstrapped = true;

  ObjectDefineProperties(globalThis, windowOrWorkerGlobalScope);
  ObjectDefineProperties(globalThis, mainRuntimeGlobalProperties);
  ObjectSetPrototypeOf(globalThis, Window.prototype);

  if (runtimeOptions.inspectFlag) {
    const consoleFromDeno = globalThis.console;
    wrapConsole(consoleFromDeno, consoleFromV8);
  }

  event.setEventTargetData(globalThis);
  event.saveGlobalThisReference(globalThis);

  runtimeStart(runtimeOptions);

  // delete `Deno` global
  delete globalThis.Deno;

  util.log("args", runtimeOptions.args);
}

ObjectDefineProperties(globalThis, {
  bootstrap: {
    value: {
      mainRuntime: bootstrapMainRuntime,
    },
    configurable: true,
  },
});
