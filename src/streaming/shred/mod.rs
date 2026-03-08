// ShredStream modules
pub mod connection;
pub mod pool;
pub mod types;

// Re-export main types
pub use connection::*;
pub use pool::*;
pub use types::*;

// Re-export from common module
pub use crate::streaming::common::{
    ConnectionConfig, MetricsEventType, MetricsManager, PerformanceMetrics, StreamClientConfig,
};
