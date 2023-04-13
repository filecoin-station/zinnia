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
// https://github.com/bajtos/rust-libp2p-ping-poc/blob/v1/src/peer/behaviour.rs

pub use super::handler::{ProtocolInfo, RequestPayload, ResponsePayload};

use super::handler::{RequestProtocol, RequestResponseHandler, RequestResponseHandlerEvent};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    task::{Context, Poll},
    time::Duration,
};

use libp2p::core::{ConnectedPoint, Endpoint, Multiaddr};
use libp2p::identity::PeerId;
use libp2p::swarm::{
    behaviour::{AddressChange, ConnectionClosed, ConnectionEstablished, DialFailure, FromSwarm},
    dial_opts::DialOpts,
    NetworkBehaviour, NotifyHandler, PollParameters, ToSwarm,
};
use libp2p::swarm::{ConnectionDenied, ConnectionId, THandler, THandlerInEvent, THandlerOutEvent};

use smallvec::SmallVec;

/// An inbound request or response.
#[derive(Debug)]
pub enum RequestResponseMessage {
    /// A response message.
    Response {
        /// The ID of the request that produced this response.
        ///
        /// See [`RequestResponse::send_request`].
        request_id: RequestId,
        /// The response message.
        response: ResponsePayload,
    },
}

/// The events emitted by a [`RequestResponse`] protocol.
#[derive(Debug)]
pub enum RequestResponseEvent {
    /// An incoming message (request or response).
    Message {
        /// The peer who sent the message.
        peer: PeerId,
        /// The incoming message.
        message: RequestResponseMessage,
    },
    /// An outbound request failed.
    OutboundFailure {
        /// The peer to whom the request was sent.
        peer: PeerId,
        /// The (local) ID of the failed request.
        request_id: RequestId,
        /// The error that occurred.
        error: OutboundFailure,
    },
    /// An inbound request failed.
    InboundFailure {
        /// The peer from whom the request was received.
        peer: PeerId,
        /// The error that occurred.
        error: InboundFailure,
    },
}

/// Possible failures occurring in the context of sending
/// an outbound request and receiving the response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboundFailure {
    /// The request could not be sent because a dialing attempt failed.
    DialFailure,
    /// The request timed out before a response was received.
    ///
    /// It is not known whether the request may have been
    /// received (and processed) by the remote peer.
    Timeout,
    /// The connection closed before a response was received.
    ///
    /// It is not known whether the request may have been
    /// received (and processed) by the remote peer.
    ConnectionClosed,
    /// The remote supports none of the requested protocols.
    UnsupportedProtocols,
}

impl fmt::Display for OutboundFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutboundFailure::DialFailure => {
                write!(f, "Failed to dial the requested peer")
            }
            OutboundFailure::Timeout => {
                write!(f, "Timeout while waiting for a response")
            }
            OutboundFailure::ConnectionClosed => {
                write!(f, "Connection was closed before a response was received")
            }
            OutboundFailure::UnsupportedProtocols => {
                write!(f, "The remote supports none of the requested protocols")
            }
        }
    }
}

impl std::error::Error for OutboundFailure {}

/// Possible failures occurring in the context of receiving an
/// inbound request and sending a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundFailure {
    /// The inbound request timed out, either while reading the
    /// incoming request or before a response is sent.
    /// We don't support inbound requests yet, therefore this error
    /// should never happen in practice.
    Timeout,
    /// The local peer supports none of the protocols requested
    /// by the remote.
    UnsupportedProtocols,
}

impl fmt::Display for InboundFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InboundFailure::Timeout => {
                write!(f, "Timeout while receiving request or sending response")
            }
            InboundFailure::UnsupportedProtocols => write!(
                f,
                "The local peer supports none of the protocols requested by the remote"
            ),
        }
    }
}

impl std::error::Error for InboundFailure {}

/// The ID of an inbound or outbound request.
///
/// Note: [`RequestId`]'s uniqueness is only guaranteed between two
/// inbound and likewise between two outbound requests. There is no
/// uniqueness guarantee in a set of both inbound and outbound
/// [`RequestId`]s nor in a set of inbound or outbound requests
/// originating from different [`RequestResponse`] behaviours.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The configuration for a `RequestResponse` protocol.
#[derive(Debug, Clone)]
pub struct RequestResponseConfig {
    pub request_timeout: Duration,
    pub connection_keep_alive: Duration,
}

impl Default for RequestResponseConfig {
    fn default() -> Self {
        Self {
            connection_keep_alive: Duration::from_secs(10),
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// A request/response protocol for some message codec.
pub struct RequestResponse {
    /// The next (local) request ID.
    next_request_id: RequestId,
    /// The protocol configuration.
    config: RequestResponseConfig,
    /// Pending events to return from `poll`.
    pending_events: VecDeque<ToSwarm<RequestResponseEvent, RequestProtocol>>,
    /// The currently connected peers, their pending outbound and inbound responses and their known,
    /// reachable addresses, if any.
    connected: HashMap<PeerId, SmallVec<[Connection; 2]>>,
    /// Externally managed addresses via `add_address` and `remove_address`.
    addresses: HashMap<PeerId, SmallVec<[Multiaddr; 6]>>,
    /// Requests that have not yet been sent and are waiting for a connection
    /// to be established.
    pending_outbound_requests: HashMap<PeerId, SmallVec<[RequestProtocol; 10]>>,
}

impl RequestResponse {
    /// Creates a new `RequestResponse` behaviour for the given
    /// codec and configuration.
    pub fn new(cfg: RequestResponseConfig) -> Self {
        RequestResponse {
            next_request_id: RequestId(1),
            config: cfg,
            pending_events: VecDeque::new(),
            connected: HashMap::new(),
            pending_outbound_requests: HashMap::new(),
            addresses: HashMap::new(),
        }
    }

    /// Initiates sending a request.
    ///
    /// If the targeted peer is currently not connected, a dialing
    /// attempt is initiated and the request is sent as soon as a
    /// connection is established.
    ///
    /// > **Note**: In order for such a dialing attempt to succeed,
    /// > the `RequestResonse` protocol must either be embedded
    /// > in another `NetworkBehaviour` that provides peer and
    /// > address discovery, or known addresses of peers must be
    /// > managed via [`RequestResponse::add_address`] and
    /// > [`RequestResponse::remove_address`].
    pub fn send_request(
        &mut self,
        peer: &PeerId,
        protocols: &[ProtocolInfo],
        request: RequestPayload,
    ) -> RequestId {
        let request_id = self.next_request_id();
        let request = RequestProtocol {
            request_id,
            protocols: protocols.into(),
            payload: request,
        };

        if let Some(request) = self.try_send_request(peer, request) {
            self.pending_events.push_back(ToSwarm::Dial {
                opts: DialOpts::peer_id(*peer).build(),
            });
            self.pending_outbound_requests
                .entry(*peer)
                .or_default()
                .push(request);
        }

        request_id
    }

    /// Adds a known address for a peer that can be used for
    /// dialing attempts by the `Swarm`, i.e. is returned
    /// by [`NetworkBehaviour::addresses_of_peer`].
    ///
    /// Addresses added in this way are only removed by `remove_address`.
    pub fn add_address(&mut self, peer: &PeerId, address: Multiaddr) {
        self.addresses.entry(*peer).or_default().push(address);
    }

    /// Removes an address of a peer previously added via `add_address`.
    #[allow(dead_code)]
    pub fn remove_address(&mut self, peer: &PeerId, address: &Multiaddr) {
        let mut last = false;
        if let Some(addresses) = self.addresses.get_mut(peer) {
            addresses.retain(|a| a != address);
            last = addresses.is_empty();
        }
        if last {
            self.addresses.remove(peer);
        }
    }

    /// Checks whether a peer is currently connected.
    #[allow(dead_code)]
    pub fn is_connected(&self, peer: &PeerId) -> bool {
        if let Some(connections) = self.connected.get(peer) {
            !connections.is_empty()
        } else {
            false
        }
    }

    /// Checks whether an outbound request to the peer with the provided
    /// [`PeerId`] initiated by [`RequestResponse::send_request`] is still
    /// pending, i.e. waiting for a response.
    #[allow(dead_code)]
    pub fn is_pending_outbound(&self, peer: &PeerId, request_id: &RequestId) -> bool {
        // Check if request is already sent on established connection.
        let est_conn = self
            .connected
            .get(peer)
            .map(|cs| {
                cs.iter()
                    .any(|c| c.pending_inbound_responses.contains(request_id))
            })
            .unwrap_or(false);
        // Check if request is still pending to be sent.
        let pen_conn = self
            .pending_outbound_requests
            .get(peer)
            .map(|rps| rps.iter().any(|rp| rp.request_id == *request_id))
            .unwrap_or(false);

        est_conn || pen_conn
    }

    /// Returns the next request ID.
    fn next_request_id(&mut self) -> RequestId {
        let request_id = self.next_request_id;
        self.next_request_id.0 += 1;
        request_id
    }

    /// Tries to send a request by queueing an appropriate event to be
    /// emitted to the `Swarm`. If the peer is not currently connected,
    /// the given request is return unchanged.
    fn try_send_request(
        &mut self,
        peer: &PeerId,
        request: RequestProtocol,
    ) -> Option<RequestProtocol> {
        if let Some(connections) = self.connected.get_mut(peer) {
            if connections.is_empty() {
                return Some(request);
            }
            let ix = (request.request_id.0 as usize) % connections.len();
            let conn = &mut connections[ix];
            conn.pending_inbound_responses.insert(request.request_id);
            self.pending_events.push_back(ToSwarm::NotifyHandler {
                peer_id: *peer,
                handler: NotifyHandler::One(conn.id),
                event: request,
            });
            None
        } else {
            Some(request)
        }
    }

    /// Remove pending inbound response for the given peer and connection.
    ///
    /// Returns `true` if the provided connection to the given peer is still
    /// alive and the [`RequestId`] was previously present and is now removed.
    /// Returns `false` otherwise.
    fn remove_pending_inbound_response(
        &mut self,
        peer: &PeerId,
        connection: ConnectionId,
        request: &RequestId,
    ) -> bool {
        self.get_connection_mut(peer, connection)
            .map(|c| c.pending_inbound_responses.remove(request))
            .unwrap_or(false)
    }

    /// Returns a mutable reference to the connection in `self.connected`
    /// corresponding to the given [`PeerId`] and [`ConnectionId`].
    fn get_connection_mut(
        &mut self,
        peer: &PeerId,
        connection: ConnectionId,
    ) -> Option<&mut Connection> {
        self.connected
            .get_mut(peer)
            .and_then(|connections| connections.iter_mut().find(|c| c.id == connection))
    }

    fn on_address_change(
        &mut self,
        AddressChange {
            peer_id,
            connection_id,
            new,
            ..
        }: AddressChange,
    ) {
        let new_address = match new {
            ConnectedPoint::Dialer { address, .. } => Some(address.clone()),
            ConnectedPoint::Listener { .. } => None,
        };
        let connections = self
            .connected
            .get_mut(&peer_id)
            .expect("Address change can only happen on an established connection.");

        let connection = connections
            .iter_mut()
            .find(|c| c.id == connection_id)
            .expect("Address change can only happen on an established connection.");
        connection.address = new_address;
    }

    fn on_connection_established(
        &mut self,
        ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            other_established,
            ..
        }: ConnectionEstablished,
    ) {
        let address = match endpoint {
            ConnectedPoint::Dialer { address, .. } => Some(address.clone()),
            ConnectedPoint::Listener { .. } => None,
        };
        self.connected
            .entry(peer_id)
            .or_default()
            .push(Connection::new(connection_id, address));

        if other_established == 0 {
            if let Some(pending) = self.pending_outbound_requests.remove(&peer_id) {
                for request in pending {
                    let request = self.try_send_request(&peer_id, request);
                    assert!(request.is_none());
                }
            }
        }
    }

    fn on_connection_closed(
        &mut self,
        ConnectionClosed {
            peer_id,
            connection_id,
            remaining_established,
            ..
        }: ConnectionClosed<<Self as NetworkBehaviour>::ConnectionHandler>,
    ) {
        let connections = self
            .connected
            .get_mut(&peer_id)
            .expect("Expected some established connection to peer before closing.");

        let connection = connections
            .iter()
            .position(|c| c.id == connection_id)
            .map(|p: usize| connections.remove(p))
            .expect("Expected connection to be established before closing.");

        debug_assert_eq!(connections.is_empty(), remaining_established == 0);
        if connections.is_empty() {
            self.connected.remove(&peer_id);
        }

        for request_id in connection.pending_inbound_responses {
            self.pending_events.push_back(ToSwarm::GenerateEvent(
                RequestResponseEvent::OutboundFailure {
                    peer: peer_id,
                    request_id,
                    error: OutboundFailure::ConnectionClosed,
                },
            ));
        }
    }

    fn on_dial_failure(&mut self, DialFailure { peer_id, .. }: DialFailure) {
        if let Some(peer) = peer_id {
            // If there are pending outgoing requests when a dial failure occurs,
            // it is implied that we are not connected to the peer, since pending
            // outgoing requests are drained when a connection is established and
            // only created when a peer is not connected when a request is made.
            // Thus these requests must be considered failed, even if there is
            // another, concurrent dialing attempt ongoing.
            if let Some(pending) = self.pending_outbound_requests.remove(&peer) {
                for request in pending {
                    self.pending_events.push_back(ToSwarm::GenerateEvent(
                        RequestResponseEvent::OutboundFailure {
                            peer,
                            request_id: request.request_id,
                            error: OutboundFailure::DialFailure,
                        },
                    ));
                }
            }
        }
    }
}

impl NetworkBehaviour for RequestResponse {
    type ConnectionHandler = RequestResponseHandler;
    type OutEvent = RequestResponseEvent;

    fn handle_pending_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        _addresses: &[Multiaddr],
        _effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        let peer = match maybe_peer {
            None => return Ok(vec![]),
            Some(peer) => peer,
        };

        let mut addresses = Vec::new();
        if let Some(connections) = self.connected.get(&peer) {
            addresses.extend(connections.iter().filter_map(|c| c.address.clone()))
        }
        if let Some(more) = self.addresses.get(&peer) {
            addresses.extend(more.into_iter().cloned());
        }
        Ok(addresses)
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(RequestResponseHandler::new(
            self.config.connection_keep_alive,
            self.config.request_timeout,
        ))
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionEstablished(connection_established) => {
                self.on_connection_established(connection_established)
            }
            FromSwarm::ConnectionClosed(connection_closed) => {
                self.on_connection_closed(connection_closed)
            }
            FromSwarm::AddressChange(address_change) => self.on_address_change(address_change),
            FromSwarm::DialFailure(dial_failure) => self.on_dial_failure(dial_failure),
            FromSwarm::ListenFailure(_) => {}
            FromSwarm::NewListener(_) => {}
            FromSwarm::NewListenAddr(_) => {}
            FromSwarm::ExpiredListenAddr(_) => {}
            FromSwarm::ListenerError(_) => {}
            FromSwarm::ListenerClosed(_) => {}
            FromSwarm::NewExternalAddr(_) => {}
            FromSwarm::ExpiredExternalAddr(_) => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        peer: PeerId,
        connection: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        match event {
            RequestResponseHandlerEvent::Response {
                request_id,
                response,
            } => {
                let removed = self.remove_pending_inbound_response(&peer, connection, &request_id);
                debug_assert!(
                    removed,
                    "Expect request_id to be pending before receiving response.",
                );

                let message = RequestResponseMessage::Response {
                    request_id,
                    response,
                };
                self.pending_events.push_back(ToSwarm::GenerateEvent(
                    RequestResponseEvent::Message { peer, message },
                ));
            }
            RequestResponseHandlerEvent::OutboundTimeout(request_id) => {
                let removed = self.remove_pending_inbound_response(&peer, connection, &request_id);
                debug_assert!(
                    removed,
                    "Expect request_id to be pending before request times out."
                );

                self.pending_events.push_back(ToSwarm::GenerateEvent(
                    RequestResponseEvent::OutboundFailure {
                        peer,
                        request_id,
                        error: OutboundFailure::Timeout,
                    },
                ));
            }
            RequestResponseHandlerEvent::InboundTimeout => {
                // Note: `RequestResponseHandlerEvent::InboundTimeout` is emitted both for timing
                // out to receive the request and for timing out sending the response. In the former
                // case the request is never added to `pending_outbound_responses` and thus one can
                // not assert the request_id to be present before removing it.
                // self.remove_pending_outbound_response(&peer, connection, request_id);

                self.pending_events.push_back(ToSwarm::GenerateEvent(
                    RequestResponseEvent::InboundFailure {
                        peer,
                        error: InboundFailure::Timeout,
                    },
                ));
            }
            RequestResponseHandlerEvent::OutboundUnsupportedProtocols(request_id) => {
                let removed = self.remove_pending_inbound_response(&peer, connection, &request_id);
                debug_assert!(
                    removed,
                    "Expect request_id to be pending before failing to connect.",
                );

                self.pending_events.push_back(ToSwarm::GenerateEvent(
                    RequestResponseEvent::OutboundFailure {
                        peer,
                        request_id,
                        error: OutboundFailure::UnsupportedProtocols,
                    },
                ));
            }
            RequestResponseHandlerEvent::InboundUnsupportedProtocols => {
                // Note: No need to call `self.remove_pending_outbound_response`,
                // `RequestResponseHandlerEvent::Request` was never emitted for this request and
                // thus request was never added to `pending_outbound_responses`.
                self.pending_events.push_back(ToSwarm::GenerateEvent(
                    RequestResponseEvent::InboundFailure {
                        peer,
                        error: InboundFailure::UnsupportedProtocols,
                    },
                ));
            }
        }
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> {
        if let Some(ev) = self.pending_events.pop_front() {
            return Poll::Ready(ev);
        } else if self.pending_events.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.pending_events.shrink_to_fit();
        }

        Poll::Pending
    }
}

/// Internal threshold for when to shrink the capacity
/// of empty queues. If the capacity of an empty queue
/// exceeds this threshold, the associated memory is
/// released.
pub const EMPTY_QUEUE_SHRINK_THRESHOLD: usize = 100;

/// Internal information tracked for an established connection.
struct Connection {
    id: ConnectionId,
    address: Option<Multiaddr>,
    /// Pending inbound responses for previously sent requests on this
    /// connection.
    pending_inbound_responses: HashSet<RequestId>,
}

impl Connection {
    fn new(id: ConnectionId, address: Option<Multiaddr>) -> Self {
        Self {
            id,
            address,
            pending_inbound_responses: Default::default(),
        }
    }
}
