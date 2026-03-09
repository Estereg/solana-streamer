use crate::{
    common::AnyResult,
    streaming::{
        grpc::{pool::factory, EventPretty},
        yellowstone_grpc::{TransactionFilter, YellowstoneGrpc},
    },
};
use futures::{SinkExt, StreamExt};
use log::error;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestPing,
};

use crate::streaming::common::SubscriptionHandle;

const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

#[derive(Debug)]
pub enum SystemEvent {
    NewTransfer(Box<TransferInfo>),
    Error(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferInfo {
    pub slot: u64,
    pub signature: String,
    pub tx: Option<SubscribeUpdateTransactionInfo>,
}

impl YellowstoneGrpc {
    ///
    /// # Errors
    /// Returns error if gRPC connection fails.
    pub async fn subscribe_system<F>(
        &self,
        callback: F,
        account_include: Option<Vec<String>>,
        account_exclude: Option<Vec<String>>,
    ) -> AnyResult<()>
    where
        F: Fn(SystemEvent) + Send + Sync + Clone + 'static,
    {
        // Stop any existing subscription first
        self.stop().await;

        let addrs = vec![SYSTEM_PROGRAM_ID.to_string()];
        let account_include = account_include.unwrap_or_default();
        let account_exclude = account_exclude.unwrap_or_default();
        let tx_filter =
            vec![TransactionFilter { account_include, account_exclude, account_required: addrs }];
        let transactions = self.subscription_manager.get_subscribe_request_filter(&tx_filter, None);
        let (mut subscribe_tx, mut stream, _) = self
            .subscription_manager
            .subscribe_with_request(transactions, None, None, None)
            .await?;

        let stream_handle = tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        let created_at = msg.created_at;
                        match msg.update_oneof {
                            Some(UpdateOneof::Transaction(transaction_update)) => {
                                match factory::create_transaction_pretty(
                                    transaction_update,
                                    created_at,
                                ) {
                                    Ok(transaction_pretty) => {
                                        let event_pretty =
                                            EventPretty::Transaction(Box::new(transaction_pretty));
                                        Self::process_system_transaction(
                                            event_pretty,
                                            &callback,
                                        );
                                    }
                                    Err(e) => {
                                        error!("Failed to parse transaction update: {e:?}");
                                    }
                                }
                            }
                            Some(UpdateOneof::Ping(_)) => {
                                let _ = subscribe_tx
                                    .send(SubscribeRequest {
                                        ping: Some(SubscribeRequestPing { id: 1 }),
                                        ..Default::default()
                                    })
                                    .await;
                            }
                            _ => {
                                // Pong or other message types, ignore for system subscription
                            }
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // Save subscription handle so it can be stopped later
        let subscription_handle = SubscriptionHandle::new(stream_handle, None, None);
        *self.subscription_handle.lock().await = Some(subscription_handle);

        Ok(())
    }

    /// Process a system transaction event
    fn process_system_transaction<F>(event_pretty: EventPretty, callback: &F)
    where
        F: Fn(SystemEvent) + Send + Sync,
    {
        if let EventPretty::Transaction(transaction_pretty) = event_pretty {
            callback(SystemEvent::NewTransfer(Box::new(TransferInfo {
                slot: transaction_pretty.slot,
                signature: transaction_pretty.signature.to_string(),
                tx: Some(transaction_pretty.grpc_tx),
            })));
        }
    }
}
