use std::time::Duration;

pub use libp2p::ping::Config as PingConfig;

use super::behaviour::RequestResponseConfig;

#[derive(Debug, Clone)]
pub struct PeerNodeConfig {
    pub request_timeout: Duration,
    pub connection_keep_alive: Duration,
    pub ping: PingConfig,
}

impl Default for PeerNodeConfig {
    fn default() -> Self {
        Self {
            connection_keep_alive: Duration::from_secs(10),
            request_timeout: Duration::from_secs(10),
            ping: Default::default(),
        }
    }
}

impl PeerNodeConfig {
    pub fn request_response_config(&self) -> RequestResponseConfig {
        RequestResponseConfig {
            request_timeout: self.request_timeout,
            connection_keep_alive: self.connection_keep_alive,
        }
    }

    pub fn ping_config(&self) -> PingConfig {
        self.ping.clone()
    }
}
