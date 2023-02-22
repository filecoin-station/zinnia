"use strict";

((window) => {
  const core = window.Deno.core;
  const ops = core.ops;

  const { ObjectDefineProperties, ObjectCreate, ObjectFreeze } =
    window.__bootstrap.primordials;

  window.__bootstrap.libp2p ??= {};

  window.__bootstrap.libp2p.defaultPeerProps = {
    peerId: {
      get() {
        return ops.op_p2p_get_peer_id();
      },
      enumerable: true,
      configurable: true,
    },
  };
})(globalThis);
