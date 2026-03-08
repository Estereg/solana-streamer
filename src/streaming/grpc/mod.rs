// gRPC modules
pub mod pool;
pub mod subscription;
pub mod types;

// Re-export main types
pub use pool::*;
pub use subscription::*;
pub use types::*;

// Re-export from common module
pub use crate::streaming::common::{
    ConnectionConfig, MetricsManager, PerformanceMetrics, StreamClientConfig as ClientConfig,
};
