use solana_streamer_sdk::streaming::{
    event_parser::{Protocol, DexEvent},
    shred::StreamClientConfig,
    ShredStreamGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ShredStream Streamer...");
    test_shreds().await?;
    Ok(())
}

async fn test_shreds() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to ShredStream events...");

    // Create low-latency configuration
    // Enable performance monitoring, has performance overhead, disabled by default
    let config = StreamClientConfig {
        enable_metrics: true,
        ..Default::default()
    };
    let shred_stream =
        ShredStreamGrpc::new_with_config("http://127.0.0.1:10800".to_string(), config).await?;

    let callback = create_event_callback();
    let protocols = vec![
        Protocol::PumpFun,
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
        Protocol::RaydiumAmmV4,
    ];

    // Event filtering
    // No event filtering, includes all events
    let event_type_filter = None;
    // Only include PumpSwapBuy events and PumpSwapSell events
    // let event_type_filter =
    //     EventTypeFilter { include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] };

    println!("Listening for events, press Ctrl+C to stop...");
    shred_stream.shredstream_subscribe(protocols, event_type_filter, callback).await?;

    // Support stop method, test code - stop after 1000 seconds asynchronously
    let shred_clone = shred_stream.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        shred_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(DexEvent) {
    |event: DexEvent| {
        let event_type = event.metadata().event_type.clone();
        let tx_index = event.metadata().tx_index;
        println!(
            "🎉 Event received! Type: {event_type:?}, tx_index: {tx_index:?}"
        );
        if let DexEvent::BlockMetaEvent(e) = event {
            let handle_us = e.metadata.handle_us;
            println!("BlockMetaEvent: {handle_us:?}");
        }
    }
}
