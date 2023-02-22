use std::cell::RefCell;
use std::rc::Rc;

use deno_core::anyhow::{anyhow, Context, Result};
use deno_core::error::AnyError;
use deno_core::{include_js_files, op, Extension, OpState, ZeroCopyBuf};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use peer::{PeerNode, PeerNodeConfig};

mod peer;

#[derive(Clone, Debug, Default)]
pub struct Options {
  /// Configuration options for the built-in (default) peer node
  default_peer: PeerNodeConfig,
}

// Next:
// - Create Deno ops - dial, request_protocol
// - Where to store the peer_node instance?
// - Export a Deno extension
// - Add this new extension to our runtime
// - Write some JS tests

#[derive(Debug, Clone, Copy)]
struct DefaultNodeResourceId(deno_core::ResourceId);

pub fn init(options: Options) -> Extension {
  Extension::builder(env!("CARGO_PKG_NAME"))
    .js(include_js_files!(
      prefix "internal:ext/libp2p",
      "js/01_peer.js",
    ))
    .ops(vec![
      op_p2p_get_peer_id::decl(),
      op_p2p_request_protocol::decl(),
    ])
    .state(move |state| {
      let default_node = PeerNode::spawn(options.default_peer.clone())
        // FIXME: map errors to AnyError instead of panicking
        // We need to convert `Box<dyn Error + Send>` to `anyhow::Error`
        .unwrap();
      let rid = state.resource_table.add(default_node);
      state.put::<DefaultNodeResourceId>(DefaultNodeResourceId(rid));
      Ok(())
    })
    .build()
}

#[op]
pub fn op_p2p_get_peer_id(state: &mut OpState) -> Result<String> {
  let rid = state.borrow::<DefaultNodeResourceId>().0;
  let node = state.resource_table.get::<PeerNode>(rid)?;
  let id = node.peer_id();
  Ok(id.to_string())
}

#[op]
pub async fn op_p2p_request_protocol(
  state: Rc<RefCell<OpState>>,
  remote_address: String,
  protocol_name: String,
  request_payload: ZeroCopyBuf,
) -> Result<Vec<u8>> {
  let mut peer_addr: Multiaddr = remote_address
    .parse()
    .with_context(|| "invalid remote address")?;

  let peer_id = match peer_addr.pop() {
    Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash)
      .map_err(|_multihash| anyhow!("Invalid peer ID multihash")),
    _ => Err(anyhow!("remote address must contain a valid peer ID")),
  }?;

  let rid = state.borrow().borrow::<DefaultNodeResourceId>().0;
  let node = state.borrow().resource_table.get::<PeerNode>(rid)?;

  let response_payload = node
    .request_protocol(
      peer_id,
      peer_addr,
      protocol_name.as_bytes(),
      request_payload.to_vec(),
    )
    .await
    // FIXME: find how to convert `Box<dyn Error + Send>` to `anyhow::Error`
    .map_err(|err| anyhow!("cannot dial remote peer: {}", err))?;

  Ok(response_payload)
}

pub async fn shutdown(_state: Rc<RefCell<OpState>>) -> Result<(), AnyError> {
  // FIXME: shutdown the default PeerNode
  // Note: the code bellow does not work because `node` is not mutable
  // let rid = state.borrow::<DefaultNodeResourceId>().0;
  // let node = state.resource_table.get::<PeerNode>(rid)?;
  // node.shutdown().await?;
  Ok(())
}
