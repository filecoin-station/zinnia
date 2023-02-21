// ZINNIA VERSION: Copyright 2023 Protocol Labs. All rights reserved. MIT OR Apache-2.0 license.
// ORIGINAL WORK: Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// https://github.com/denoland/deno/blob/34bfa2cb2c1f0f74a94ced8fc164e81cc91cb9f4/runtime/js/98_global_scope.js

"use strict";

((window) => {
  const core = Deno.core;

  const util = window.__bootstrap.util;
  const event = window.__bootstrap.event;
  const eventTarget = window.__bootstrap.eventTarget;
  const timers = window.__bootstrap.timers;
  const base64 = window.__bootstrap.base64;
  const encoding = window.__bootstrap.encoding;
  const Console = window.__bootstrap.console.Console;
  const compression = window.__bootstrap.compression;
  const performance = window.__bootstrap.performance;
  const url = window.__bootstrap.url;
  const urlPattern = window.__bootstrap.urlPattern;
  const streams = window.__bootstrap.streams;
  const fileReader = window.__bootstrap.fileReader;
  const file = window.__bootstrap.file;
  const fetch = window.__bootstrap.fetch;
  const messagePort = window.__bootstrap.messagePort;
  const webidl = window.__bootstrap.webidl;
  const domException = window.__bootstrap.domException;
  const abortSignal = window.__bootstrap.abortSignal;
  const globalInterfaces = window.__bootstrap.globalInterfaces;

  // https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope
  const windowOrWorkerGlobalScope = {
    AbortController: util.nonEnumerable(abortSignal.AbortController),
    AbortSignal: util.nonEnumerable(abortSignal.AbortSignal),
    // Intentionally disabled until we need this.
    // Blob: util.nonEnumerable(file.Blob),
    ByteLengthQueuingStrategy: util.nonEnumerable(
      streams.ByteLengthQueuingStrategy,
    ),
    CloseEvent: util.nonEnumerable(event.CloseEvent),
    CompressionStream: util.nonEnumerable(compression.CompressionStream),
    CountQueuingStrategy: util.nonEnumerable(streams.CountQueuingStrategy),
    // TODO https://github.com/filecoin-station/zinnia/issues/33
    // CryptoKey: util.nonEnumerable(crypto.CryptoKey),
    CustomEvent: util.nonEnumerable(event.CustomEvent),
    DecompressionStream: util.nonEnumerable(compression.DecompressionStream),
    DOMException: util.nonEnumerable(domException.DOMException),
    ErrorEvent: util.nonEnumerable(event.ErrorEvent),
    Event: util.nonEnumerable(event.Event),
    EventTarget: util.nonEnumerable(eventTarget.EventTarget),
    // Intentionally disabled until we need this.
    // File: util.nonEnumerable(file.File),
    // FileReader: util.nonEnumerable(fileReader.FileReader),
    // TODO:  https://github.com/filecoin-station/zinnia/issues/25
    // FormData: util.nonEnumerable(formData.FormData),
    // Headers: util.nonEnumerable(headers.Headers),
    MessageEvent: util.nonEnumerable(event.MessageEvent),
    Performance: util.nonEnumerable(performance.Performance),
    PerformanceEntry: util.nonEnumerable(performance.PerformanceEntry),
    PerformanceMark: util.nonEnumerable(performance.PerformanceMark),
    PerformanceMeasure: util.nonEnumerable(performance.PerformanceMeasure),
    PromiseRejectionEvent: util.nonEnumerable(event.PromiseRejectionEvent),
    // TODO:  https://github.com/filecoin-station/zinnia/issues/25
    // ProgressEvent: util.nonEnumerable(event.ProgressEvent),
    ReadableStream: util.nonEnumerable(streams.ReadableStream),
    ReadableStreamDefaultReader: util.nonEnumerable(
      streams.ReadableStreamDefaultReader,
    ),
    Request: util.nonEnumerable(fetch.Request),
    Response: util.nonEnumerable(fetch.Response),
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
    WritableStreamDefaultWriter: util.nonEnumerable(
      streams.WritableStreamDefaultWriter,
    ),
    WritableStreamDefaultController: util.nonEnumerable(
      streams.WritableStreamDefaultController,
    ),
    ReadableByteStreamController: util.nonEnumerable(
      streams.ReadableByteStreamController,
    ),
    ReadableStreamBYOBReader: util.nonEnumerable(
      streams.ReadableStreamBYOBReader,
    ),
    ReadableStreamBYOBRequest: util.nonEnumerable(
      streams.ReadableStreamBYOBRequest,
    ),
    ReadableStreamDefaultController: util.nonEnumerable(
      streams.ReadableStreamDefaultController,
    ),
    TransformStreamDefaultController: util.nonEnumerable(
      streams.TransformStreamDefaultController,
    ),
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
    console: util.nonEnumerable(
      new Console((msg, level) => core.print(msg, level > 1)),
    ),
    // TODO: https://github.com/filecoin-station/zinnia/issues/33
    // crypto: util.readOnly(crypto.crypto),
    // Crypto: util.nonEnumerable(crypto.Crypto),
    // SubtleCrypto: util.nonEnumerable(crypto.SubtleCrypto),
    fetch: util.writable(fetch.fetch),
    performance: util.writable(performance.performance),
    reportError: util.writable(event.reportError),
    setInterval: util.writable(timers.setInterval),
    setTimeout: util.writable(timers.setTimeout),
    structuredClone: util.writable(messagePort.structuredClone),
    // Branding as a WebIDL object
    [webidl.brand]: util.nonEnumerable(webidl.brand),
  };

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
  };

  window.__bootstrap.globalScope = {
    windowOrWorkerGlobalScope,
    mainRuntimeGlobalProperties,
  };
})(this);
