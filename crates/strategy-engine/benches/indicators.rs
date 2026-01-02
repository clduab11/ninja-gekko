use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use strategy_engine::indicators::prelude::*;
use strategy_engine::strategies::MomentumStrategy;

fn bench_rsi_update(c: &mut Criterion) {
    let mut rsi = Rsi::new(14);

    // Warmup
    for i in 1..=14 {
        rsi.update(Decimal::from(i * 10));
    }

    c.bench_function("rsi_update", |b| {
        b.iter(|| rsi.update(black_box(dec!(150.25))))
    });
}

fn bench_momentum_strategy_cycle(c: &mut Criterion) {
    // Simulates the indicator part of the strategy
    let mut state = strategy_engine::indicators::state::IndicatorState::new(200);
    state.add(Rsi::new(14));
    state.add(Ema::new(9));
    state.add(Ema::new(21));

    // Warmup
    for i in 1..=200 {
        state.update(Candle {
            open: dec!(100),
            high: dec!(105),
            low: dec!(95),
            close: dec!(102),
            volume: dec!(1000),
            timestamp: i,
        });
    }

    let candle = Candle {
        open: dec!(150),
        high: dec!(152),
        low: dec!(148),
        close: dec!(151),
        volume: dec!(1500),
        timestamp: 1000,
    };

    c.bench_function("momentum_strategy_cycle", |b| {
        b.iter(|| state.update(black_box(candle.clone())))
    });
}

criterion_group!(benches, bench_rsi_update, bench_momentum_strategy_cycle);
criterion_main!(benches);
