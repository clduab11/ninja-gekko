use std::time::Instant;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use event_bus::{
    EventBusBuilder, EventMetadata, EventSender, Priority, PublishMode, SignalEvent,
    SignalEventPayload, StrategySignal,
};
use ninja_gekko_core::types::{OrderSide, OrderType};
use rust_decimal::Decimal;
use tokio::runtime::Runtime;
use uuid::Uuid;

struct ChannelHarness {
    _bus: event_bus::EventBus,
    sender: EventSender<SignalEvent>,
    drain_handle: tokio::task::JoinHandle<()>,
}

fn setup_harness() -> ChannelHarness {
    let bus = EventBusBuilder::default().build();
    let sender = bus.signal_sender();
    let receiver = bus.signal_receiver();

    let drain_handle = tokio::spawn(async move {
        let receiver = receiver;
        while receiver.recv_async().await.is_ok() {}
    });

    ChannelHarness {
        _bus: bus,
        sender,
        drain_handle,
    }
}

fn bench_signal_dispatch(c: &mut Criterion) {
    let runtime = Runtime::new().expect("tokio runtime");

    c.bench_function("signal_dispatch_sub_millisecond", |b| {
        b.to_async(&runtime).iter_batched(
            setup_harness,
            |harness| async move {
                let iterations = 128u32;
                let start = Instant::now();
                for _ in 0..iterations {
                    let metadata = EventMetadata::new("bench.signal", Priority::Normal);
                    let payload = SignalEventPayload {
                        strategy_id: Uuid::new_v4(),
                        account_id: "bench-acct".to_string(),
                        priority: Priority::Normal,
                        signal: StrategySignal {
                            exchange: None,
                            symbol: "BTC-USD".to_string(),
                            side: OrderSide::Buy,
                            order_type: OrderType::Market,
                            quantity: Decimal::new(1, 0),
                            limit_price: None,
                            confidence: 0.5,
                            metadata: Default::default(),
                        },
                    };
                    let event = SignalEvent::new(metadata, payload);
                    harness
                        .sender
                        .publish(event, PublishMode::Blocking)
                        .unwrap();
                }
                drop(harness.sender);
                harness.drain_handle.await.unwrap();
                start.elapsed()
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_signal_dispatch);
criterion_main!(benches);
