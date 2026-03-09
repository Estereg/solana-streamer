use futures::{channel::mpsc, sink::Sink, Stream};
use std::collections::HashMap;
use tonic::{transport::channel::ClientTlsConfig, Status};
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterTransactions, SubscribeUpdate,
};

use super::types::AccountsFilterMap;
use super::types::TransactionsFilterMap;
use crate::common::AnyResult;
use crate::streaming::common::StreamClientConfig as ClientConfig;
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::yellowstone_grpc::AccountFilter;
use crate::streaming::yellowstone_grpc::TransactionFilter;

/// Subscription manager
#[derive(Clone)]
pub struct SubscriptionManager {
    endpoint: String,
    x_token: Option<String>,
    config: ClientConfig,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    #[must_use]
    pub const fn new(endpoint: String, x_token: Option<String>, config: ClientConfig) -> Self {
        Self { endpoint, x_token, config }
    }

    /// Create gRPC connection
    ///
    /// # Errors
    ///
    /// Returns error if client construction or connection fails.
    pub async fn connect(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(self.config.connection.max_decoding_message_size)
            .connect_timeout(self.config.connection.connect_timeout)
            .timeout(self.config.connection.request_timeout);
        Ok(builder.connect().await?)
    }

    /// Create subscription request and return stream
    ///
    /// # Errors
    ///
    /// Returns error if connection or subscription fails.
    pub async fn subscribe_with_request(
        &self,
        transactions: Option<TransactionsFilterMap>,
        accounts: Option<AccountsFilterMap>,
        commitment: Option<CommitmentLevel>,
        event_type_filter: Option<&EventTypeFilter>,
    ) -> AnyResult<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
        SubscribeRequest,
    )> {
        #[allow(clippy::zero_sized_map_values)]
        let blocks_meta = if event_type_filter.is_some_and(|f| !f.include_block_event()) {
            HashMap::new()
        } else {
            HashMap::from([(String::new(), SubscribeRequestFilterBlocksMeta {})])
        };

        let subscribe_request = SubscribeRequest {
            accounts: accounts.unwrap_or_default(),
            transactions: transactions.unwrap_or_default(),
            blocks_meta,
            commitment: Some(commitment.unwrap_or(CommitmentLevel::Processed) as i32),
            ..Default::default()
        };
        let mut client = self.connect().await?;
        let (sink, stream) = client.subscribe_with_request(Some(subscribe_request.clone())).await?;
        Ok((sink, stream, subscribe_request))
    }

    /// Create account subscription request and return stream
    #[must_use]
    pub fn subscribe_with_account_request(
        &self,
        account_filter: &[AccountFilter],
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<AccountsFilterMap> {
        if let Some(f) = event_type_filter {
            if !f.include_account_event() {
                return None;
            }
        }
        if account_filter.is_empty() {
            return None;
        }
        let mut accounts = HashMap::with_capacity(account_filter.len());
        for (index, af) in account_filter.iter().enumerate() {
            accounts.insert(
                format!("account_{index}"),
                SubscribeRequestFilterAccounts {
                    account: af.account.clone(),
                    owner: af.owner.clone(),
                    filters: af.filters.clone(),
                    nonempty_txn_signature: None,
                },
            );
        }
        Some(accounts)
    }

    /// Generate subscription request filter
    #[must_use]
    pub fn get_subscribe_request_filter(
        &self,
        transaction_filter: &[TransactionFilter],
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<TransactionsFilterMap> {
        if let Some(f) = event_type_filter {
            if !f.include_transaction_event() {
                return None;
            }
        }
        let mut transactions = HashMap::with_capacity(transaction_filter.len());
        for (index, tf) in transaction_filter.iter().enumerate() {
            transactions.insert(
                format!("transaction_{index}"),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    signature: None,
                    account_include: tf.account_include.clone(),
                    account_exclude: tf.account_exclude.clone(),
                    account_required: tf.account_required.clone(),
                },
            );
        }
        Some(transactions)
    }

    /// Get configuration
    #[must_use]
    pub const fn get_config(&self) -> &ClientConfig {
        &self.config
    }
}
