const primordials = globalThis.__bootstrap.primordials;
const { ObjectDefineProperties, ObjectCreate } = primordials;

import * as libp2p from "ext:zinnia_libp2p/01_peer.js";

const zinniaNs = ObjectCreate(null);
ObjectDefineProperties(zinniaNs, libp2p.defaultPeerProps);

export { zinniaNs };
