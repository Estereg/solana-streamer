// ShredStream modules
pub mod connection;
pub mod types;

// Re-export main types
pub use connection::*;
pub use types::*;

// Re-export from common module
pub use crate::streaming::common::{
    ConnectionConfig, MetricCategory, MetricsManager, PerformanceMetrics, StreamClientConfig,
};
