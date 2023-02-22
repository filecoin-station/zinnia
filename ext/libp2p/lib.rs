use deno_core::error::AnyError;
use deno_core::{include_js_files, op, Extension};

mod peer;

// Next:
// - Create Deno ops - dial, request_protocol
// - Where to store the peer_node instance?
// - Export a Deno extension
// - Add this new extension to our runtime
// - Write some JS tests

pub fn init() -> Extension {
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
      // TODO: put the peer instance into the state
      Ok(())
    })
    .build()
}

#[op]
pub fn op_p2p_get_peer_id() -> Result<String, AnyError> {
  Ok("123".into())
}
