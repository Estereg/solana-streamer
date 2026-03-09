use crate::common::AnyResult;
use crate::streaming::common::MetricCategory;
use crate::streaming::event_parser::common::{ParseOptions, TransactionParams};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::core::account_event_parser::AccountEventParser;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::core::event_parser::EventParser;
use crate::streaming::event_parser::{core::traits::DexEvent, Protocol};
use crate::streaming::grpc::{EventPretty, MetricsManager};
use crate::streaming::shred::TransactionWithSlot;
use std::sync::Arc;

/// Shared metrics callback that wraps a user callback with latency tracking
#[inline]
fn metrics_callback(
    event: &DexEvent,
    callback: &Arc<dyn Fn(DexEvent) + Send + Sync>,
) {
    let metadata = event.metadata();
    #[allow(clippy::cast_precision_loss)]
    let processing_time_us = metadata.handle_us as f64;
    let recv_us = metadata.recv_us;
    let block_time_ms = metadata.block_time_ms;

    callback(event.clone());

    update_metrics_with_latency(
        MetricCategory::Transaction,
        1,
        processing_time_us,
        recv_us,
        block_time_ms,
    );
}

/// Process GRPC transaction events
///
/// # Errors
///
/// Returns error if transaction parsing fails.
pub async fn process_grpc_transaction(
    event_pretty: EventPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> AnyResult<()> {
    match event_pretty {
        EventPretty::Account(account_pretty) => {
            MetricsManager::global().add_account_process_count();

            let account_event = AccountEventParser::parse_account_event(
                protocols,
                &account_pretty,
                event_type_filter,
            );

            if let Some(event) = account_event {
                #[allow(clippy::cast_precision_loss)]
                let processing_time_us = event.metadata().handle_us as f64;
                callback(event);
                update_metrics(MetricCategory::Account, 1, processing_time_us);
            }
        }
        EventPretty::Transaction(transaction_pretty) => {
            MetricsManager::global().add_tx_process_count();

            let slot = transaction_pretty.slot;
            let signature = transaction_pretty.signature;
            let block_time = transaction_pretty.block_time;
            let recv_us = transaction_pretty.recv_us;
            let tx_index = transaction_pretty.tx_index;
            let grpc_tx = transaction_pretty.grpc_tx;

            let mut adapter_callback = |event: &DexEvent| {
                metrics_callback(event, &callback);
            };

            let options = ParseOptions {
                protocols,
                event_type_filter,
            };

            let params = TransactionParams {
                signature,
                slot,
                block_time,
                recv_us,
                tx_index,
                recent_blockhash: None,
            };

            EventParser::parse_grpc_transaction(
                options,
                grpc_tx,
                params,
                &mut adapter_callback,
            )?;
        }
        EventPretty::BlockMeta(block_meta_pretty) => {
            MetricsManager::global().add_block_meta_process_count();

            let block_time_ms = block_meta_pretty
                .block_time
                .map_or_else(|| {
                    i64::try_from(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()).unwrap_or(i64::MAX)
                }, |ts| ts.seconds * 1000 + i64::from(ts.nanos) / 1_000_000);

            let block_meta_event = CommonEventParser::generate_block_meta_event(
                block_meta_pretty.slot,
                block_meta_pretty.block_hash,
                block_time_ms,
                block_meta_pretty.recv_us,
            );

            #[allow(clippy::cast_precision_loss)]
            let processing_time_us = block_meta_event.metadata().handle_us as f64;
            callback(block_meta_event);
            update_metrics(MetricCategory::BlockMeta, 1, processing_time_us);
        }
    }

    Ok(())
}

/// Process Shred transaction events
///
/// # Errors
///
/// Returns error if transaction parsing fails.
pub async fn process_shred_transaction(
    transaction_with_slot: TransactionWithSlot,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> AnyResult<()> {
    MetricsManager::global().add_tx_process_count();

    let transaction = transaction_with_slot.transaction;
    let slot = transaction_with_slot.slot;
    let tx_index = transaction_with_slot.tx_index;

    if transaction.signatures.is_empty() {
        return Ok(());
    }

    let signature = transaction.signatures[0];
    let recv_us = transaction_with_slot.recv_us;

    let mut adapter_callback = |event: &DexEvent| {
        metrics_callback(event, &callback);
    };

    // Shred path only gets static_account_keys, no inner_instructions. See docs/SHREDSTREAM_LIMITATIONS.md
    // If tx uses ALT, accounts might be default/wrong; no CPI merge, timestamp/reserves mostly 0.
    let accounts = transaction.message.static_account_keys();

    let options = ParseOptions {
        protocols,
        event_type_filter,
    };

    let params = TransactionParams {
        signature,
        slot,
        block_time: None,
        recv_us,
        tx_index,
        recent_blockhash: None,
    };

    EventParser::parse_instruction_events_from_versioned_transaction(
        options,
        &transaction,
        params,
        accounts,
        &[],
        &mut adapter_callback,
    )?;

    Ok(())
}

/// Update metrics for event processing
#[inline]
fn update_metrics(ty: MetricCategory, count: u64, time_us: f64) {
    MetricsManager::global().update_metrics(ty, count, time_us);
}

/// Update metrics with latency check
#[inline]
fn update_metrics_with_latency(
    ty: MetricCategory,
    count: u64,
    time_us: f64,
    recv_us: i64,
    block_time_ms: i64,
) {
    MetricsManager::global().update_metrics_with_latency(ty, count, time_us, recv_us, block_time_ms);
}
