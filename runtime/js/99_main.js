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
import * as util from "ext:zinnia_runtime/06_util.js";
import * as event from "ext:deno_web/02_event.js";
import * as timers from "ext:deno_web/02_timers.js";
import {
  getDefaultInspectOptions,
  getNoColor,
  inspectArgs,
  quoteString,
  setNoColorFn,
} from "ext:deno_console/01_console.js";
import * as performance from "ext:deno_web/15_performance.js";
import {
  mainRuntimeGlobalProperties,
  windowOrWorkerGlobalScope,
} from "ext:zinnia_runtime/98_global_scope.js";
import { setLassieConfig } from "ext:zinnia_runtime/fetch.js";
import { setVersions } from "ext:zinnia_runtime/90_zinnia_apis.js";

function formatException(error) {
  if (ObjectPrototypeIsPrototypeOf(ErrorPrototype, error)) {
    return null;
  } else if (typeof error == "string") {
    return `Uncaught ${inspectArgs([quoteString(error, getDefaultInspectOptions())], {
      colors: !getNoColor(),
    })}`;
  } else {
    return `Uncaught ${inspectArgs([error], { colors: !getNoColor() })}`;
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
  // FIXME: rework to lazy load, see
  // https://github.com/denoland/deno/commit/1ef617e8f3d48098e69e222b6eb6fe981aeca1c3
  setNoColorFn(() => runtimeOptions.noColor || !runtimeOptions.isTty);

  setLassieConfig(runtimeOptions.lassieUrl, runtimeOptions.lassieAuth);
  setVersions(runtimeOptions.zinniaVersion, runtimeOptions.v8Version);
}

let hasBootstrapped = false;
// Set up global properties shared by main and worker runtime.
ObjectDefineProperties(globalThis, windowOrWorkerGlobalScope);

function bootstrapMainRuntime(runtimeOptions) {
  if (hasBootstrapped) {
    throw new Error("Worker runtime already bootstrapped");
  }

  performance.setTimeOrigin(DateNow());

  // Remove bootstrapping data from the global scope
  delete globalThis.__bootstrap;
  delete globalThis.bootstrap;
  hasBootstrapped = true;

  ObjectDefineProperties(globalThis, mainRuntimeGlobalProperties);
  ObjectSetPrototypeOf(globalThis, Window.prototype);

  if (runtimeOptions.inspectFlag) {
    const consoleFromV8 = core.console;
    const consoleFromDeno = globalThis.console;
    core.wrapConsole(consoleFromDeno, consoleFromV8);
  }

  event.setEventTargetData(globalThis);
  event.saveGlobalThisReference(globalThis);

  runtimeStart(runtimeOptions);

  ObjectDefineProperties(globalThis.Zinnia, {
    walletAddress: util.readOnly(runtimeOptions.walletAddress),
  });

  // delete `Deno` global
  delete globalThis.Deno;

  util.log("args", runtimeOptions.args);
}

globalThis.bootstrap = {
  mainRuntime: bootstrapMainRuntime,
};

// Workaround to silence Deno runtime assert
// "Following modules were not evaluated; make sure they are imported from other code"
import "ext:zinnia_runtime/internals.js";
import "ext:zinnia_runtime/test.js";
import "ext:zinnia_runtime/vendored/asserts.bundle.js";
