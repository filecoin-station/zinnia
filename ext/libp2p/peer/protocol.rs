//! The definition of a request/response protocol via outbound substream
//! upgrades. The outbound upgrade sends a request and receives a response.

// This code is very loosely based on request-response protocol in rust-libp2p
// https://github.com/libp2p/rust-libp2p/blob/v0.50.0/protocols/request-response/src/handler/protocol.rs
//
// See the following file for the history of changes:
// https://github.com/bajtos/rust-libp2p-ping-poc/blob/v1/src/peer/handler/protocol.rs

use libp2p::core::upgrade::{OutboundUpgrade, UpgradeInfo};
use libp2p::futures::{future::BoxFuture, prelude::*};
use libp2p::swarm::NegotiatedSubstream;
use smallvec::SmallVec;

use std::{fmt, io};

use super::behaviour::RequestId;

// FIXME: Can we use `[u8]` instead? How to avoid cloning when sending the data between threads?
pub type RequestPayload = Vec<u8>;
pub type ResponsePayload = Vec<u8>;

pub type ProtocolInfo = SmallVec<[u8; 16]>;

/// Request substream upgrade protocol.
///
/// Sends a request and receives a response.
pub struct RequestProtocol {
    pub(crate) protocols: SmallVec<[ProtocolInfo; 2]>,
    pub(crate) request_id: RequestId,
    pub(crate) payload: RequestPayload,
}

impl fmt::Debug for RequestProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestProtocol")
            .field("request_id", &self.request_id)
            .field("protocols", &self.protocols)
            .field("payload", &self.payload)
            .finish()
    }
}

impl UpgradeInfo for RequestProtocol {
    type Info = ProtocolInfo;
    type InfoIter = smallvec::IntoIter<[Self::Info; 2]>;

    fn protocol_info(&self) -> Self::InfoIter {
        self.protocols.clone().into_iter()
    }
}

impl OutboundUpgrade<NegotiatedSubstream> for RequestProtocol {
    type Output = ResponsePayload;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, mut io: NegotiatedSubstream, _protocol: Self::Info) -> Self::Future {
        async move {
            // 1. Write the request payload
            io.write_all(&self.payload).await?;
            io.flush().await?;

            // 2. Signal the end of request substream
            io.close().await?;

            // 3. Read back the response - at most 10 MB
            let mut response: ResponsePayload = Default::default();
            io.take(10 * 1024 * 1024).read_to_end(&mut response).await?;
            Ok(response)
        }
        .boxed()
    }
}
