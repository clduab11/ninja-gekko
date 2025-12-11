use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use data_pipeline::ingestion::RawMarketMessage;
use data_pipeline::normalizer::MarketNormalizer;
use exchange_connectors::{ExchangeId, MarketTick, StreamMessage};
use rust_decimal::Decimal;
use tokio::runtime::Runtime;

fn bench_normalizer(c: &mut Criterion) {
    let rt = Runtime::new().expect("runtime");
    c.bench_function("normalize_tick", |b| {
        b.to_async(&rt).iter_batched(
            || MarketNormalizer::new(),
            |mut normalizer| async move {
                let tick = StreamMessage::Tick(MarketTick {
                    symbol: "BTC-USD".into(),
                    bid: Decimal::new(30_000, 0),
                    ask: Decimal::new(30_001, 0),
                    last: Decimal::new(30_000, 0),
                    volume_24h: Decimal::new(100, 0),
                    timestamp: chrono::Utc::now(),
                });
                let raw: RawMarketMessage = (ExchangeId::Kraken, tick);
                normalizer.normalize(raw)
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_normalizer);
criterion_main!(benches);
