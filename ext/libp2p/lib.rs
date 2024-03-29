use std::cell::RefCell;
use std::rc::Rc;

use deno_core::anyhow::{anyhow, Context, Result};
use deno_core::error::AnyError;
use deno_core::{op2, JsBuffer, OpState};
use libp2p::identity::PeerId;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use peer::PeerNode;

pub use peer::PeerNodeConfig;

mod peer;

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Configuration options for the built-in (default) peer node
    pub default_peer: PeerNodeConfig,
}

#[derive(Debug, Clone, Copy)]
struct DefaultNodeResourceId(deno_core::ResourceId);

deno_core::extension!(
    zinnia_libp2p,
    ops = [
        op_p2p_get_peer_id,
        op_p2p_request_protocol,
    ],
    esm = [
        dir "js",
        "01_peer.js",
    ],
    options = {
        default_peer: PeerNodeConfig,
    },
    state = |state, options| {
        let default_node = PeerNode::spawn(options.default_peer)
            // FIXME: map errors to AnyError instead of panicking
            // We need to convert `Box<dyn Error + Send>` to `anyhow::Error`
            .unwrap();
        let rid = state.resource_table.add(default_node);
        state.put::<DefaultNodeResourceId>(DefaultNodeResourceId(rid));
    },
);

#[op2]
#[string]
pub fn op_p2p_get_peer_id(state: &mut OpState) -> Result<String> {
    let rid = state.borrow::<DefaultNodeResourceId>().0;
    let node = state.resource_table.get::<PeerNode>(rid)?;
    let id = node.peer_id();
    Ok(id.to_string())
}

#[op2(async)]
#[buffer]
pub async fn op_p2p_request_protocol(
    state: Rc<RefCell<OpState>>,
    #[string] remote_address: String,
    #[string] protocol_name: String,
    #[buffer] request_payload: JsBuffer,
) -> Result<Vec<u8>> {
    let mut peer_addr: Multiaddr = remote_address
        .parse()
        .with_context(|| "invalid remote address")?;

    let peer_id = match peer_addr.pop() {
        Some(Protocol::P2p(hash)) => {
            PeerId::from_multihash(hash).map_err(|_multihash| anyhow!("Invalid peer ID multihash"))
        }
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
