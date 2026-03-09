use anyhow::Result;
use solana_streamer_sdk::streaming::event_parser::common::filter::EventTypeFilter;
use solana_streamer_sdk::streaming::event_parser::common::types::EventType;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::streaming::yellowstone_grpc::{
    AccountFilter, TransactionFilter, YellowstoneGrpc,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::sleep;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

const PUMPFUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

const GRPC_ENDPOINT: &str = "https://solana-yellowstone-grpc.publicnode.com:443";
const API_KEY: Option<&str> = None;
const MONITORING_DURATION_SECS: u64 = 10;

/// Demonstrates dynamic subscription updates and filter changes in real-time
#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to Yellowstone gRPC at {GRPC_ENDPOINT}");
    let client =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(ToString::to_string))?);

    let event_counter = Arc::new(AtomicU64::new(0));
    let counter = event_counter.clone();

    let callback =
        move |event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            let count_plus_1 = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let protocol = event.metadata().protocol.to_string();
            println!("Event #{count_plus_1}: {protocol:11} - {:.8}...", event.metadata().signature);
        };

    println!("\n=== Phase 1: PumpFun only ===");
    let pumpfun_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    let account_filter = AccountFilter { account: vec![], owner: vec![], filters: vec![] };
    let trade_event_filter = EventTypeFilter {
        include: vec![
            EventType::PumpFunBuy,
            EventType::PumpFunSell,
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
        ],
    };

    if let Err(e) = client
        .subscribe_events_immediate(
            vec![Protocol::PumpFun, Protocol::RaydiumCpmm],
            &[pumpfun_filter],
            &[account_filter],
            Some(trade_event_filter),
            None,
            callback,
        )
        .await
    {
        println!("Failed to create subscription: {e}");
        return Ok(());
    }

    println!(
        "Subscribed to PumpFun transactions with trade event filters, monitoring for {MONITORING_DURATION_SECS}s...",
    );
    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    let phase1_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 1: {phase1_count} events");

    println!("\n=== Phase 2: PumpFun + RaydiumCpmm ===");
    let multi_protocol_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string(), RAYDIUM_CPMM_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            &[multi_protocol_filter],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {e}");
        return Ok(());
    }

    println!(
        "Updated to PumpFun + RaydiumCpmm transactions, monitoring for {MONITORING_DURATION_SECS}s...",
    );
    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    let phase2_count = event_counter.load(Ordering::Relaxed);
    let events_in_phase2 = phase2_count - phase1_count;
    println!("Phase 2: {events_in_phase2} events");

    println!("\n=== Phase 3: RaydiumCpmm only ===");
    let raydium_cpmm_filter = TransactionFilter {
        account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            &[raydium_cpmm_filter],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {e}");
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to RaydiumCpmm transactions only, monitoring for {MONITORING_DURATION_SECS}s...",
    );
    let phase3_count = event_counter.load(Ordering::Relaxed);
    let events_in_phase3 = phase3_count - phase2_count;
    println!("Phase 3: {events_in_phase3} events");

    println!("\n=== Phase 4: Back to PumpFun only ===");
    let pumpfun_only_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            &[pumpfun_only_filter],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {e}");
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to PumpFun transactions only, monitoring for {MONITORING_DURATION_SECS}s...",
    );
    let phase4_count = event_counter.load(Ordering::Relaxed);
    let events_in_phase4 = phase4_count - phase3_count;
    println!("Phase 4: {events_in_phase4} events");

    println!("\n=== Phase 5: All events ===");
    let empty_filter = TransactionFilter {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            &[empty_filter],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {e}");
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to all transactions (no filters), monitoring for {MONITORING_DURATION_SECS}s...",
    );
    let phase5_count = event_counter.load(Ordering::Relaxed);
    let events_in_phase5 = phase5_count - phase4_count;
    println!("Phase 5: {events_in_phase5} events");

    println!("\n=== Phase 6: Silence ===");

    let random_keypair_1 = Keypair::new();
    let random_keypair_2 = Keypair::new();
    let random_pubkey_1 = random_keypair_1.pubkey();
    let random_pubkey_2 = random_keypair_2.pubkey();

    let silence_filter = TransactionFilter {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![random_pubkey_1.to_string(), random_pubkey_2.to_string()],
    };

    if let Err(e) = client
        .update_subscription(
            &[silence_filter],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {e}");
        return Ok(());
    }

    println!("Updated to random addresses (expecting silence), monitoring for 3s...");
    let before_silence = event_counter.load(Ordering::Relaxed);
    let start_time = Instant::now();
    let last_event_time = Arc::new(Mutex::new(start_time));
    let last_event_time_clone = last_event_time.clone();

    let mut last_count = before_silence;
    for _ in 0..6 {
        sleep(Duration::from_millis(500)).await;
        let current_count = event_counter.load(Ordering::Relaxed);
        if current_count > last_count {
            if let Ok(mut time) = last_event_time_clone.lock() {
                *time = Instant::now();
            }
            last_count = current_count;
        }
    }

    let final_count = event_counter.load(Ordering::Relaxed);
    let events_during_silence = final_count - before_silence;

    if events_during_silence == 0 {
        println!("Phase 6: 0 events (immediate filter application)");
    } else if let Ok(last_time) = last_event_time.lock() {
        let propagation_time = last_time.duration_since(start_time);
        let propagation_time_ms = propagation_time.as_millis();
        println!(
            "Phase 6: {events_during_silence} events during propagation, filter took {propagation_time_ms}ms"
        );
    }

    println!("\n=== Phase 7: Shutdown ===");

    let shutdown_client =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(ToString::to_string))?);

    let shutdown_event_counter = Arc::new(AtomicU64::new(0));
    let shutdown_counter = shutdown_event_counter.clone();
    let shutdown_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            shutdown_counter.fetch_add(1, Ordering::Relaxed);
        };

    if let Err(e) = shutdown_client
        .subscribe_events_immediate(
            vec![Protocol::PumpFun, Protocol::RaydiumCpmm],
            &[TransactionFilter {
                account_include: vec![],
                account_exclude: vec![],
                account_required: vec![],
            }],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            shutdown_callback,
        )
        .await
    {
        println!("Failed to subscribe shutdown client: {e}");
        return Ok(());
    }

    sleep(Duration::from_millis(1000)).await;
    let pre_stop_count = shutdown_event_counter.load(Ordering::Relaxed);
    println!("Received {pre_stop_count} events before stop");

    let stop_time = Instant::now();
    shutdown_client.stop().await;
    let shutdown_duration = stop_time.elapsed();
    println!("stop() completed in {:.1}ms", shutdown_duration.as_millis());

    let post_stop_count = shutdown_event_counter.load(Ordering::Relaxed);
    let during_stop = post_stop_count - pre_stop_count;
    if during_stop > 0 {
        println!("  {during_stop} events received during stop()");
    }

    let last_event_time = Arc::new(Mutex::new(stop_time));
    let last_event_time_clone = last_event_time.clone();
    let mut last_count = post_stop_count;

    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let current_count = shutdown_event_counter.load(Ordering::Relaxed);
        if current_count > last_count {
            if let Ok(mut time) = last_event_time_clone.lock() {
                *time = Instant::now();
            }
            last_count = current_count;
        }
    }

    let final_count = shutdown_event_counter.load(Ordering::Relaxed);
    let after_stop = final_count - post_stop_count;

    if after_stop == 0 {
        println!("Phase 7: Clean shutdown - no events after stop()");
    } else if let Ok(last_time) = last_event_time.lock() {
        let post_stop_duration = last_time.duration_since(stop_time);
        let silence_duration = Instant::now().duration_since(*last_time);
        let post_stop_duration_ms = post_stop_duration.as_millis();
        let silence_duration_ms = silence_duration.as_millis();
        println!(
            "Phase 7: {after_stop} events arrived up to {post_stop_duration_ms}ms after stop(), then silent for {silence_duration_ms}ms",
        );
    }

    println!("\n=== Subscription enforcement ===");

    let test_callback =
        |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {};

    match client
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            &[TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            test_callback,
        )
        .await
    {
        Ok(()) => println!("ERROR: Same client created second subscription"),
        Err(e) if e.to_string().contains("Already subscribed") => {
            println!("✓ Single subscription enforcement working");
        }
        Err(e) => println!("Unexpected error: {e}"),
    }

    let client2 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(std::string::ToString::to_string))?);

    let client2_counter = Arc::new(AtomicU64::new(0));
    let counter2 = client2_counter.clone();
    let client2_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            counter2.fetch_add(1, Ordering::Relaxed);
        };

    match client2
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            &[TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            client2_callback,
        )
        .await
    {
        Ok(()) => {
            sleep(Duration::from_millis(500)).await;
            let count = client2_counter.load(Ordering::Relaxed);
            println!("✓ Second client: {count} events");
            client2.stop().await;
        }
        Err(e) => println!("ERROR: Second client failed: {e}"),
    }

    println!("\n=== Advanced subscription enforcement ===");

    let test_callback_advanced =
        |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {};

    let client3 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(ToString::to_string))?);

    // First subscription should succeed
    match client3
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            &[TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            test_callback_advanced,
        )
        .await
    {
        Ok(()) => {
            // Second subscription attempt on same client should fail
            match client3
                .subscribe_events_immediate(
                    vec![Protocol::RaydiumCpmm],
                    &[TransactionFilter {
                        account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                        account_exclude: vec![],
                        account_required: vec![],
                    }],
                    &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
                    None,
                    None,
                    |_| {},
                )
                .await
            {
                Ok(()) => println!("ERROR: Same client created second advanced subscription"),
                Err(e) if e.to_string().contains("Already subscribed") => {
                    println!("✓ Advanced single subscription enforcement working");
                }
                Err(e) => println!("Unexpected error: {e}"),
            }
        }
        Err(e) => println!("ERROR: First advanced subscription failed: {e}"),
    }

    // Test that a second client can subscribe using advanced method
    let client4 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(ToString::to_string))?);

    let client4_counter = Arc::new(AtomicU64::new(0));
    let counter4 = client4_counter.clone();
    let client4_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            counter4.fetch_add(1, Ordering::Relaxed);
        };

    match client4
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            &[TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            &[AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            client4_callback,
        )
        .await
    {
        Ok(()) => {
            sleep(Duration::from_millis(500)).await;
            let count = client4_counter.load(Ordering::Relaxed);
            println!("✓ Second client (advanced): {count} events");
            client4.stop().await;
        }
        Err(e) => println!("ERROR: Second client (advanced) failed: {e}"),
    }

    client3.stop().await;

    client.stop().await;

    Ok(())
}
