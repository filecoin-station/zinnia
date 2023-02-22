use deno_core::error::AnyError;
use deno_core::{include_js_files, op, Extension, OpState};
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

pub fn init(options: Options) -> Extension {
  Extension::builder(env!("CARGO_PKG_NAME"))
    .js(include_js_files!(
      prefix "internal:ext/libp2p",
      "js/01_peer.js",
    ))
    .ops(vec![
      op_p2p_get_peer_id::decl(),
      // TODO
    ])
    .state(move |state| {
      state.put::<PeerNode>(
        PeerNode::spawn(options.default_peer.clone()).unwrap(),
      );
      Ok(())
    })
    .build()
}

#[op]
pub fn op_p2p_get_peer_id(state: &mut OpState) -> Result<String, AnyError> {
  let peer = state.borrow::<PeerNode>();
  let id = peer.peer_id();
  Ok(id.to_string())
}
