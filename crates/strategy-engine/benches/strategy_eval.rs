use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;
use uuid::Uuid;
use wat::parse_str as parse_wat;

use strategy_engine::{
    sandbox::{WasmStrategyConfig, WasmStrategyModule},
    traits::{MarketSnapshot, StrategyContext},
};

const TEST_WASM: &str = r#"(module
  (import "host" "log" (func $log (param i32 i32)))
  (import "host" "emit_signal" (func $emit (param i32 i32)))
  (memory (export "memory") 1)
  (global $next (mut i32) (i32.const 1024))
  (data (i32.const 0) "{\"strategy_id\":\"00000000-0000-0000-0000-000000000000\",\"account_id\":\"sandbox-account\",\"priority\":\"High\",\"signal\":{\"exchange\":null,\"symbol\":\"BTC-USD\",\"side\":\"Buy\",\"order_type\":\"Market\",\"quantity\":\"1\",\"limit_price\":null,\"confidence\":1.0,\"metadata\":{}}}")
  (data (i32.const 512) "log")
  (func (export "alloc") (param $size i32) (result i32)
        (local $ptr i32)
        (local.set $ptr (global.get $next))
        (global.set $next (i32.add (local.get $ptr) (local.get $size)))
        (local.get $ptr))
  (func (export "evaluate") (param $ctx_ptr i32) (param $ctx_len i32) (result i32)
        (call $log (i32.const 512) (i32.const 3))
        (call $emit (i32.const 0) (i32.const 249))
        (i32.const 0)))"#;

fn wasm_eval_benchmark(c: &mut Criterion) {
    let wasm_bytes = parse_wat(TEST_WASM).expect("valid wasm");
    let module =
        WasmStrategyModule::from_bytes(&wasm_bytes, &WasmStrategyConfig::default()).unwrap();
    let mut instance = module.instantiate(WasmStrategyConfig::default()).unwrap();

    let account_id = String::from("sandbox-account");
    let snapshots = [MarketSnapshot {
        symbol: "BTC-USD".into(),
        bid: Decimal::from(30_000u32),
        ask: Decimal::from(30_010u32),
        last: Decimal::from(30_005u32),
        timestamp: Utc::now(),
    }];
    let context = StrategyContext::new(&account_id, &snapshots, Uuid::nil(), Utc::now());

    c.bench_function("wasm_strategy_eval", |b| {
        b.iter(|| {
            let decision = instance.evaluate(&context).unwrap();
            assert!(!decision.signals.is_empty());
        });
    });
}

criterion_group!(benches, wasm_eval_benchmark);
criterion_main!(benches);
