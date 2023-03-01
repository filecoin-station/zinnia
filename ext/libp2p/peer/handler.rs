// Copyright Protocol Labs and Zinnia contributors
//
// This code is based on libp2p with the following notice:
// https://github.com/libp2p/rust-libp2p/blob/v0.50.0/protocols/request-response/src/handler.rs
//
// Copyright 2020 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// See the following file for the history of changes:
// https://github.com/bajtos/rust-libp2p-ping-poc/blob/v1/src/peer/handler.rs

pub use super::protocol::{ProtocolInfo, RequestPayload, RequestProtocol, ResponsePayload};

use super::behaviour::{RequestId, EMPTY_QUEUE_SHRINK_THRESHOLD};
// use super::protocol::{ProtocolName};

use libp2p::core::upgrade::{DeniedUpgrade, NegotiationError, UpgradeError};
use libp2p::swarm::handler::{
    ConnectionEvent, ConnectionHandler, ConnectionHandlerEvent, ConnectionHandlerUpgrErr,
    DialUpgradeError, FullyNegotiatedOutbound, KeepAlive, ListenUpgradeError,
};
use libp2p::swarm::SubstreamProtocol;

use std::time::Instant;
use std::{
    collections::VecDeque,
    fmt, io,
    task::{Context, Poll},
    time::Duration,
};

/// A connection handler of a `RequestResponse` protocol.
#[doc(hidden)]
pub struct RequestResponseHandler {
    /// The keep-alive timeout of idle connections. A connection is considered
    /// idle if there are no outbound substreams.
    keep_alive_timeout: Duration,
    /// The timeout for inbound and outbound substreams (i.e. request
    /// and response processing).
    substream_timeout: Duration,
    /// The current connection keep-alive.
    keep_alive: KeepAlive,
    /// A pending fatal error that results in the connection being closed.
    pending_error: Option<ConnectionHandlerUpgrErr<io::Error>>,
    /// Queue of events to emit in `poll()`.
    pending_events: VecDeque<RequestResponseHandlerEvent>,
    /// Outbound upgrades waiting to be emitted as an `OutboundSubstreamRequest`.
    outbound: VecDeque<RequestProtocol>,
}

impl RequestResponseHandler {
    pub(super) fn new(keep_alive_timeout: Duration, substream_timeout: Duration) -> Self {
        Self {
            keep_alive: KeepAlive::Yes,
            keep_alive_timeout,
            substream_timeout,
            outbound: VecDeque::new(),
            pending_events: VecDeque::new(),
            pending_error: None,
        }
    }

    fn on_dial_upgrade_error(
        &mut self,
        DialUpgradeError { info, error }: DialUpgradeError<
            <Self as ConnectionHandler>::OutboundOpenInfo,
            <Self as ConnectionHandler>::OutboundProtocol,
        >,
    ) {
        match error {
            ConnectionHandlerUpgrErr::Timeout => {
                self.pending_events
                    .push_back(RequestResponseHandlerEvent::OutboundTimeout(info));
            }
            ConnectionHandlerUpgrErr::Upgrade(UpgradeError::Select(NegotiationError::Failed)) => {
                // The remote merely doesn't support the protocol(s) we requested.
                // This is no reason to close the connection, which may
                // successfully communicate with other protocols already.
                // An event is reported to permit user code to react to the fact that
                // the remote peer does not support the requested protocol(s).
                self.pending_events.push_back(
                    RequestResponseHandlerEvent::OutboundUnsupportedProtocols(info),
                );
            }
            _ => {
                // Anything else is considered a fatal error or misbehaviour of
                // the remote peer and results in closing the connection.
                self.pending_error = Some(error);
            }
        }
    }
    fn on_listen_upgrade_error(
        &mut self,
        ListenUpgradeError { info: _, error }: ListenUpgradeError<
            <Self as ConnectionHandler>::InboundOpenInfo,
            <Self as ConnectionHandler>::InboundProtocol,
        >,
    ) {
        match error {
            ConnectionHandlerUpgrErr::Timeout => self
                .pending_events
                .push_back(RequestResponseHandlerEvent::InboundTimeout),
            ConnectionHandlerUpgrErr::Upgrade(UpgradeError::Select(NegotiationError::Failed)) => {
                // The local peer merely doesn't support the protocol(s) requested.
                // This is no reason to close the connection, which may
                // successfully communicate with other protocols already.
                // An event is reported to permit user code to react to the fact that
                // the local peer does not support the requested protocol(s).
                self.pending_events
                    .push_back(RequestResponseHandlerEvent::InboundUnsupportedProtocols);
            }
            _ => {
                // Anything else is considered a fatal error or misbehaviour of
                // the remote peer and results in closing the connection.
                // self.pending_error = Some(error);
                unreachable!();
            }
        }
    }
}

/// The events emitted by the [`RequestResponseHandler`].
#[doc(hidden)]
pub enum RequestResponseHandlerEvent {
    /// A response has been received.
    Response {
        request_id: RequestId,
        response: ResponsePayload,
    },
    /// An outbound request timed out while sending the request
    /// or waiting for the response.
    OutboundTimeout(RequestId),
    /// An outbound request failed to negotiate a mutually supported protocol.
    OutboundUnsupportedProtocols(RequestId),
    /// An inbound request timed out while waiting for the request
    /// or sending the response.
    InboundTimeout,
    /// An inbound request failed to negotiate a mutually supported protocol.
    InboundUnsupportedProtocols,
}

impl fmt::Debug for RequestResponseHandlerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestResponseHandlerEvent::Response {
                request_id,
                response: _,
            } => f
                .debug_struct("RequestResponseHandlerEvent::Response")
                .field("request_id", request_id)
                .finish(),
            RequestResponseHandlerEvent::OutboundTimeout(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::OutboundTimeout")
                .field(request_id)
                .finish(),
            RequestResponseHandlerEvent::OutboundUnsupportedProtocols(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::OutboundUnsupportedProtocols")
                .field(request_id)
                .finish(),
            RequestResponseHandlerEvent::InboundTimeout => f
                .debug_tuple("RequestResponseHandlerEvent::InboundTimeout")
                .finish(),
            RequestResponseHandlerEvent::InboundUnsupportedProtocols => f
                .debug_tuple("RequestResponseHandlerEvent::InboundUnsupportedProtocols")
                .finish(),
        }
    }
}

impl ConnectionHandler for RequestResponseHandler {
    type InEvent = RequestProtocol;
    type OutEvent = RequestResponseHandlerEvent;
    type Error = ConnectionHandlerUpgrErr<io::Error>;
    type InboundProtocol = DeniedUpgrade;
    type OutboundProtocol = RequestProtocol;
    type OutboundOpenInfo = RequestId;
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(DeniedUpgrade, ()).with_timeout(self.substream_timeout)
    }

    fn on_behaviour_event(&mut self, request: Self::InEvent) {
        self.keep_alive = KeepAlive::Yes;
        self.outbound.push_back(request);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<ConnectionHandlerEvent<RequestProtocol, RequestId, Self::OutEvent, Self::Error>> {
        // Check for a pending (fatal) error.
        if let Some(err) = self.pending_error.take() {
            // The handler will not be polled again by the `Swarm`.
            return Poll::Ready(ConnectionHandlerEvent::Close(err));
        }

        // Drain pending events.
        if let Some(event) = self.pending_events.pop_front() {
            return Poll::Ready(ConnectionHandlerEvent::Custom(event));
        } else if self.pending_events.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.pending_events.shrink_to_fit();
        }

        // Emit outbound requests.
        if let Some(request) = self.outbound.pop_front() {
            let info = request.request_id;
            return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(request, info)
                    .with_timeout(self.substream_timeout),
            });
        }

        debug_assert!(self.outbound.is_empty());

        if self.outbound.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.outbound.shrink_to_fit();
        }

        if self.keep_alive.is_yes() {
            // No new inbound or outbound requests. However, we may just have
            // started the latest inbound or outbound upgrade(s), so make sure
            // the keep-alive timeout is preceded by the substream timeout.
            let until = Instant::now() + self.substream_timeout + self.keep_alive_timeout;
            self.keep_alive = KeepAlive::Until(until);
        }

        Poll::Pending
    }

    fn on_connection_event(
        &mut self,
        event: ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        match event {
            ConnectionEvent::FullyNegotiatedInbound(_fully_negotiated_inbound) => {
                unreachable!();
            }
            ConnectionEvent::FullyNegotiatedOutbound(FullyNegotiatedOutbound {
                protocol: response,
                info: request_id,
            }) => {
                self.pending_events
                    .push_back(RequestResponseHandlerEvent::Response {
                        request_id,
                        response,
                    });
            }
            ConnectionEvent::DialUpgradeError(dial_upgrade_error) => {
                self.on_dial_upgrade_error(dial_upgrade_error)
            }
            ConnectionEvent::ListenUpgradeError(listen_upgrade_error) => {
                self.on_listen_upgrade_error(listen_upgrade_error)
            }
            ConnectionEvent::AddressChange(_) => {}
        }
    }
}
