use tonic::transport::Channel;

use crate::common::AnyResult;
use crate::protos::shredstream::shredstream_proxy_client::ShredstreamProxyClient;
use crate::streaming::common::{
    MetricsManager, PerformanceMetrics, StreamClientConfig, SubscriptionHandle,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// `ShredStream` gRPC client
#[derive(Clone)]
pub struct ShredStreamGrpc {
    pub(crate) shredstream_client: ShredstreamProxyClient<Channel>,
    pub(crate) config: StreamClientConfig,
    pub(crate) subscription_handle: Arc<Mutex<Option<SubscriptionHandle>>>,
}

impl ShredStreamGrpc {
    /// Create client with default configuration
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub async fn new(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, StreamClientConfig::default()).await
    }

    /// Create client with custom configuration
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub async fn new_with_config(endpoint: String, config: StreamClientConfig) -> AnyResult<Self> {
        let shredstream_client = ShredstreamProxyClient::connect(endpoint.clone()).await?;
        MetricsManager::init(config.enable_metrics);
        Ok(Self {
            shredstream_client,
            config,
            subscription_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// Get current configuration
    #[must_use]
    pub const fn get_config(&self) -> &StreamClientConfig {
        &self.config
    }

    /// Update configuration
    pub const fn update_config(&mut self, config: StreamClientConfig) {
        self.config = config;
    }

    /// Get performance metrics
    #[must_use]
    pub fn get_metrics(&self) -> PerformanceMetrics {
        MetricsManager::global().get_metrics()
    }

    /// Enable or disable performance monitoring
    pub const fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }

    /// Print performance metrics
    pub fn print_metrics(&self) {
        MetricsManager::global().print_metrics();
    }

    /// Start auto performance monitoring task
    pub fn start_auto_metrics_monitoring(&self) {
        let _ = MetricsManager::global().start_auto_monitoring();
    }

    /// Stop current subscription
    pub async fn stop(&self) {
        let mut handle_guard = self.subscription_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop();
        }
    }
}
