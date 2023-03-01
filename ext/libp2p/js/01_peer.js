"use strict";

((window) => {
  const core = window.Deno.core;
  const ops = core.ops;

  window.__bootstrap.libp2p ??= {};

  async function requestProtocol(remoteAddress, protocolName, requestPayload) {
    if (typeof remoteAddress !== "string")
      throw new TypeError(`remoteAddress must be string (found: ${typeof remoteAddress})`);
    if (typeof protocolName !== "string")
      throw new TypeError(`protocolName must be string (found: ${typeof protocolName})`);
    if (requestPayload?.constructor !== Uint8Array) {
      const actualType = requestPayload?.constructor?.name ?? typeof requestPayload;
      throw new TypeError(`requestPayload must be Uint8Array (found: ${actualType})`);
    }

    const responsePayload = await ops.op_p2p_request_protocol(
      remoteAddress,
      protocolName,
      requestPayload,
    );

    return {
      async *[Symbol.asyncIterator]() {
        yield new Uint8Array(responsePayload);
      },
    };
  }

  window.__bootstrap.libp2p.defaultPeerProps = {
    peerId: {
      get() {
        return ops.op_p2p_get_peer_id();
      },
      enumerable: true,
      configurable: true,
    },

    requestProtocol: {
      value: requestProtocol,
      writable: false,
      enumerable: true,
      configurable: true,
    },
  };
})(globalThis);
