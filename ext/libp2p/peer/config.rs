use std::time::Duration;

use libp2p::identify;
use libp2p::identity::PublicKey;
pub use libp2p::ping::Config as PingConfig;

use super::behaviour::RequestResponseConfig;

#[derive(Debug, Clone)]
pub struct PeerNodeConfig {
    /// Name and version of the local peer implementation, similar to the
    /// `User-Agent` header in the HTTP protocol.
    ///
    /// This value will be reported via the `identify` protocol.
    pub agent_version: String,

    pub request_timeout: Duration,
    pub connection_keep_alive: Duration,

    /// Configuration for the built-in `ping` protocol
    pub ping: PingConfig,
}

impl Default for PeerNodeConfig {
    fn default() -> Self {
        Self {
            agent_version: Self::default_agent_version(),
            connection_keep_alive: Duration::from_secs(10),
            request_timeout: Duration::from_secs(10),
            ping: Default::default(),
        }
    }
}

impl PeerNodeConfig {
    pub fn default_agent_version() -> String {
        format!("zinnia-libp2p/{}", env!("CARGO_PKG_VERSION"))
    }

    pub fn request_response_config(&self) -> RequestResponseConfig {
        RequestResponseConfig {
            request_timeout: self.request_timeout,
            connection_keep_alive: self.connection_keep_alive,
        }
    }

    pub fn ping_config(&self) -> PingConfig {
        self.ping.clone()
    }

    pub fn id_config(&self, local_public_key: PublicKey) -> identify::Config {
        identify::Config::new("ipfs/1.0.0".into(), local_public_key.clone())
            .with_agent_version(self.agent_version.clone())
    }
}
