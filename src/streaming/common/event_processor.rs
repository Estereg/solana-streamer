use crate::common::AnyResult;
use crate::streaming::common::MetricsEventType;
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::core::account_event_parser::AccountEventParser;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::core::event_parser::EventParser;
use crate::streaming::event_parser::{core::traits::DexEvent, Protocol};
use crate::streaming::grpc::{EventPretty, MetricsManager};
use crate::streaming::shred::TransactionWithSlot;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// 创建带 metrics 统计的 callback 包装器
///
/// 用于 Transaction 事件处理，在调用原始 callback 的同时更新 metrics
#[inline]
fn create_metrics_callback(
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> Arc<dyn Fn(DexEvent) + Send + Sync> {
    Arc::new(move |event: DexEvent| {
        let processing_time_us = event.metadata().handle_us as f64;
        callback(event);
        MetricsManager::global().update_metrics(
            MetricsEventType::Transaction,
            1,
            processing_time_us,
        );
    })
}

/// Process GRPC transaction events
pub async fn process_grpc_transaction(
    event_pretty: EventPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    bot_wallet: Option<Pubkey>,
) -> AnyResult<()> {
    match event_pretty {
        EventPretty::Account(account_pretty) => {
            MetricsManager::global().add_account_process_count();

            let account_event = AccountEventParser::parse_account_event(
                protocols,
                account_pretty,
                event_type_filter,
            );

            if let Some(event) = account_event {
                let processing_time_us = event.metadata().handle_us as f64;
                callback(event);
                update_metrics(MetricsEventType::Account, 1, processing_time_us);
            }
        }
        EventPretty::Transaction(transaction_pretty) => {
            MetricsManager::global().add_tx_process_count();

            let slot = transaction_pretty.slot;
            let signature = transaction_pretty.signature;
            let block_time = transaction_pretty.block_time;
            let recv_us = transaction_pretty.recv_us;
            let transaction_index = transaction_pretty.transaction_index;
            let grpc_tx = transaction_pretty.grpc_tx;

            let adapter_callback = create_metrics_callback(callback.clone());

            EventParser::parse_grpc_transaction_owned(
                protocols,
                event_type_filter,
                grpc_tx,
                signature,
                Some(slot),
                block_time,
                recv_us,
                bot_wallet,
                transaction_index,
                adapter_callback,
            )
            .await?;
        }
        EventPretty::BlockMeta(block_meta_pretty) => {
            MetricsManager::global().add_block_meta_process_count();

            let block_time_ms = block_meta_pretty
                .block_time
                .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

            let block_meta_event = CommonEventParser::generate_block_meta_event(
                block_meta_pretty.slot,
                block_meta_pretty.block_hash,
                block_time_ms,
                block_meta_pretty.recv_us,
            );

            let processing_time_us = block_meta_event.metadata().handle_us as f64;
            callback(block_meta_event);
            update_metrics(MetricsEventType::BlockMeta, 1, processing_time_us);
        }
    }

    Ok(())
}

/// Process Shred transaction events
pub async fn process_shred_transaction(
    transaction_with_slot: TransactionWithSlot,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    bot_wallet: Option<Pubkey>,
) -> AnyResult<()> {
    MetricsManager::global().add_tx_process_count();

    let tx = transaction_with_slot.transaction;
    let slot = transaction_with_slot.slot;

    if tx.signatures.is_empty() {
        return Ok(());
    }

    let signature = tx.signatures[0];
    let recv_us = transaction_with_slot.recv_us;

    let adapter_callback = create_metrics_callback(callback);

    EventParser::parse_versioned_transaction_owned(
        protocols,
        event_type_filter,
        tx,
        signature,
        Some(slot),
        None,
        recv_us,
        bot_wallet,
        None,
        &[],
        adapter_callback,
    )
    .await?;

    Ok(())
}

/// Update metrics for event processing
#[inline]
fn update_metrics(ty: MetricsEventType, count: u64, time_us: f64) {
    MetricsManager::global().update_metrics(ty, count, time_us);
}
