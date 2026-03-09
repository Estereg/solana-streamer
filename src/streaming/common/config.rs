use std::time::Duration;

use super::constants::{
    DEFAULT_CONNECT_TIMEOUT, DEFAULT_MAX_DECODING_MESSAGE_SIZE, DEFAULT_REQUEST_TIMEOUT,
};

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection timeout (default: 10s)
    pub connect_timeout: Duration,
    /// Request timeout (default: 60s)
    pub request_timeout: Duration,
    /// Maximum decoding message size in bytes (default: 10MB)
    pub max_decoding_message_size: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_decoding_message_size: DEFAULT_MAX_DECODING_MESSAGE_SIZE,
        }
    }
}

/// Common client configuration
#[derive(Debug, Clone, Default)]
pub struct StreamClientConfig {
    /// Connection configuration
    pub connection: ConnectionConfig,
    /// Whether performance monitoring is enabled (default: false)
    pub enable_metrics: bool,
}

