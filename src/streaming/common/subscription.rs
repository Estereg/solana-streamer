use tokio::task::JoinHandle;

/// Subscription handle for managing and stopping subscriptions
pub struct SubscriptionHandle {
    stream: JoinHandle<()>,
    event: Option<JoinHandle<()>>,
    metrics: Option<JoinHandle<()>>,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    #[must_use]
    pub const fn new(
        stream: JoinHandle<()>,
        event: Option<JoinHandle<()>>,
        metrics: Option<JoinHandle<()>>,
    ) -> Self {
        Self { stream, event, metrics }
    }

    /// Stop subscription and abort all related tasks
    pub fn stop(self) {
        self.stream.abort();
        if let Some(handle) = self.event {
            handle.abort();
        }
        if let Some(handle) = self.metrics {
            handle.abort();
        }
    }

    /// Asynchronously wait for all tasks to complete
    ///
    /// # Errors
    ///
    /// Returns a `tokio::task::JoinError` if any of the tasks failed to join correctly.
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        let _ = self.stream.await;
        if let Some(handle) = self.event {
            let _ = handle.await;
        }
        if let Some(handle) = self.metrics {
            let _ = handle.await;
        }
        Ok(())
    }
}
