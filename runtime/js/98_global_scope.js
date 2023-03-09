// ZINNIA VERSION: Copyright 2023 Protocol Labs. All rights reserved. MIT OR Apache-2.0 license.
// ORIGINAL WORK: Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// https://github.com/denoland/deno/blob/86785f21194460d713276dca2/runtime/js/98_global_scope.js

const core = globalThis.Deno.core;
const primordials = globalThis.__bootstrap.primordials;
const { ObjectDefineProperties, ObjectCreate } = primordials;

import * as util from "internal:zinnia_runtime/js/06_util.js";
import * as event from "internal:deno_web/02_event.js";
import * as timers from "internal:deno_web/02_timers.js";
import * as base64 from "internal:deno_web/05_base64.js";
import * as encoding from "internal:deno_web/08_text_encoding.js";
import * as console from "internal:deno_console/02_console.js";
import * as compression from "internal:deno_web/14_compression.js";
import * as performance from "internal:deno_web/15_performance.js";
import * as crypto from "internal:deno_crypto/00_crypto.js";
import * as url from "internal:deno_url/00_url.js";
import * as urlPattern from "internal:deno_url/01_urlpattern.js";
import * as headers from "internal:deno_fetch/20_headers.js";
import * as streams from "internal:deno_web/06_streams.js";
import * as formData from "internal:deno_fetch/21_formdata.js";
import * as request from "internal:deno_fetch/23_request.js";
import * as response from "internal:deno_fetch/23_response.js";
import * as fetch from "internal:deno_fetch/26_fetch.js";
import * as messagePort from "internal:deno_web/13_message_port.js";
import * as webidl from "internal:deno_webidl/00_webidl.js";
import DOMException from "internal:deno_web/01_dom_exception.js";
import * as abortSignal from "internal:deno_web/03_abort_signal.js";
import * as globalInterfaces from "internal:deno_web/04_global_interfaces.js";
import * as libp2p from "internal:zinnia_libp2p/js/01_peer.js";

// https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope
const windowOrWorkerGlobalScope = {
  AbortController: util.nonEnumerable(abortSignal.AbortController),
  AbortSignal: util.nonEnumerable(abortSignal.AbortSignal),
  // Intentionally disabled until we need this.
  // Blob: util.nonEnumerable(file.Blob),
  ByteLengthQueuingStrategy: util.nonEnumerable(streams.ByteLengthQueuingStrategy),
  CloseEvent: util.nonEnumerable(event.CloseEvent),
  CompressionStream: util.nonEnumerable(compression.CompressionStream),
  CountQueuingStrategy: util.nonEnumerable(streams.CountQueuingStrategy),
  CryptoKey: util.nonEnumerable(crypto.CryptoKey),
  CustomEvent: util.nonEnumerable(event.CustomEvent),
  DecompressionStream: util.nonEnumerable(compression.DecompressionStream),
  DOMException: util.nonEnumerable(DOMException),
  ErrorEvent: util.nonEnumerable(event.ErrorEvent),
  Event: util.nonEnumerable(event.Event),
  EventTarget: util.nonEnumerable(event.EventTarget),
  // Intentionally disabled until we need this.
  // File: util.nonEnumerable(file.File),
  // FileReader: util.nonEnumerable(fileReader.FileReader),
  FormData: util.nonEnumerable(formData.FormData),
  Headers: util.nonEnumerable(headers.Headers),
  MessageEvent: util.nonEnumerable(event.MessageEvent),
  Performance: util.nonEnumerable(performance.Performance),
  PerformanceEntry: util.nonEnumerable(performance.PerformanceEntry),
  PerformanceMark: util.nonEnumerable(performance.PerformanceMark),
  PerformanceMeasure: util.nonEnumerable(performance.PerformanceMeasure),
  PromiseRejectionEvent: util.nonEnumerable(event.PromiseRejectionEvent),
  ProgressEvent: util.nonEnumerable(event.ProgressEvent),
  ReadableStream: util.nonEnumerable(streams.ReadableStream),
  ReadableStreamDefaultReader: util.nonEnumerable(streams.ReadableStreamDefaultReader),
  Request: util.nonEnumerable(request.Request),
  Response: util.nonEnumerable(response.Response),
  TextDecoder: util.nonEnumerable(encoding.TextDecoder),
  TextEncoder: util.nonEnumerable(encoding.TextEncoder),
  TextDecoderStream: util.nonEnumerable(encoding.TextDecoderStream),
  TextEncoderStream: util.nonEnumerable(encoding.TextEncoderStream),
  TransformStream: util.nonEnumerable(streams.TransformStream),
  URL: util.nonEnumerable(url.URL),
  URLPattern: util.nonEnumerable(urlPattern.URLPattern),
  URLSearchParams: util.nonEnumerable(url.URLSearchParams),
  // TODO(?): WebSocket Standard
  // WebSocket: util.nonEnumerable(webSocket.WebSocket),
  MessageChannel: util.nonEnumerable(messagePort.MessageChannel),
  MessagePort: util.nonEnumerable(messagePort.MessagePort),
  // TODO(?): Service & Web Workers
  // Worker: util.nonEnumerable(worker.Worker),
  WritableStream: util.nonEnumerable(streams.WritableStream),
  WritableStreamDefaultWriter: util.nonEnumerable(streams.WritableStreamDefaultWriter),
  WritableStreamDefaultController: util.nonEnumerable(streams.WritableStreamDefaultController),
  ReadableByteStreamController: util.nonEnumerable(streams.ReadableByteStreamController),
  ReadableStreamBYOBReader: util.nonEnumerable(streams.ReadableStreamBYOBReader),
  ReadableStreamBYOBRequest: util.nonEnumerable(streams.ReadableStreamBYOBRequest),
  ReadableStreamDefaultController: util.nonEnumerable(streams.ReadableStreamDefaultController),
  TransformStreamDefaultController: util.nonEnumerable(streams.TransformStreamDefaultController),
  atob: util.writable(base64.atob),
  btoa: util.writable(base64.btoa),
  clearInterval: util.writable(timers.clearInterval),
  clearTimeout: util.writable(timers.clearTimeout),
  // TODO(?): Service & Web Workers
  // caches: {
  //   enumerable: true,
  //   configurable: true,
  //   get: caches.cacheStorage,
  // },
  // CacheStorage: util.nonEnumerable(caches.CacheStorage),
  // Cache: util.nonEnumerable(caches.Cache),
  console: util.nonEnumerable(new console.Console((msg, level) => core.print(msg, level > 1))),
  crypto: util.readOnly(crypto.crypto),
  Crypto: util.nonEnumerable(crypto.Crypto),
  SubtleCrypto: util.nonEnumerable(crypto.SubtleCrypto),
  fetch: util.writable(fetch.fetch),
  performance: util.writable(performance.performance),
  reportError: util.writable(event.reportError),
  setInterval: util.writable(timers.setInterval),
  setTimeout: util.writable(timers.setTimeout),
  structuredClone: util.writable(messagePort.structuredClone),
  // Branding as a WebIDL object
  [webidl.brand]: util.nonEnumerable(webidl.brand),
};

const zinniaNs = ObjectCreate(null);
ObjectDefineProperties(zinniaNs, libp2p.defaultPeerProps);

const mainRuntimeGlobalProperties = {
  // Location: location.locationConstructorDescriptor,
  // location: location.locationDescriptor,
  Window: globalInterfaces.windowConstructorDescriptor,
  window: util.getterOnly(() => globalThis),
  self: util.getterOnly(() => globalThis),
  // Navigator: util.nonEnumerable(Navigator),
  // navigator: util.getterOnly(() => navigator),
  // alert: util.writable(prompt.alert),
  // confirm: util.writable(prompt.confirm),
  // prompt: util.writable(prompt.prompt),
  // localStorage: util.getterOnly(webStorage.localStorage),
  // sessionStorage: util.getterOnly(webStorage.sessionStorage),
  // Storage: util.nonEnumerable(webStorage.Storage),
  Zinnia: util.readOnly(zinniaNs),
};

// prettier-ignore
export {
  windowOrWorkerGlobalScope,
  mainRuntimeGlobalProperties,
};
