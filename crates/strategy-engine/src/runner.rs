//! Strategy Runner
//!
//! Bridges the Event Bus `MarketEvent` stream to `StrategyExecutor` implementations.
//! Manages market data buffering and signal publication.

use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::Utc;
use event_bus::{
    EventBusError, EventHandler, EventSender, MarketEvent, MarketPayload, PublishMode, SignalEvent,
};
use ninja_gekko_core::types::AccountId;
use tracing::{debug, error};
use uuid::Uuid;

use crate::traits::{
    MarketSnapshot, StrategyContext, StrategyExecutor, StrategyInitContext,
};

/// Runs a strategy by feeding it market events and publishing resulting signals.
pub struct StrategyRunner<S, const N: usize> {
    strategy: S,
    signal_sender: EventSender<SignalEvent>,
    account_id: AccountId,
    snapshots: [MarketSnapshot; N],
    snapshot_index: usize,
    initialized: bool,
}

impl<S, const N: usize> StrategyRunner<S, N>
where
    S: StrategyExecutor<N> + Send + Sync,
{
    /// Create a new strategy runner
    pub fn new(
        strategy: S,
        signal_sender: EventSender<SignalEvent>,
        account_id: AccountId,
    ) -> Self {
        // Initialize snapshots with default values
        let snapshots = std::array::from_fn(|_| MarketSnapshot {
            symbol: String::new(),
            bid: rust_decimal::Decimal::ZERO,
            ask: rust_decimal::Decimal::ZERO,
            last: rust_decimal::Decimal::ZERO,
            timestamp: Utc::now(),
        });

        Self {
            strategy,
            signal_sender,
            account_id,
            snapshots,
            snapshot_index: 0,
            initialized: false,
        }
    }

    fn update_snapshots(&mut self, event: &MarketEvent) {
        if let MarketPayload::Tick { tick, .. } = event.payload() {
            // Shift snapshots to make room for new one (simple ring buffer or shift)
            // For simplicity in this implementation, we'll shift everything down
            // and put the new one at the end.
            // A ring buffer would be more efficient but requires StrategyContext to handle it.
            // Since N is small (e.g. 8), shifting is fine.
            
            for i in 0..N - 1 {
                self.snapshots[i] = self.snapshots[i + 1].clone();
            }

            self.snapshots[N - 1] = MarketSnapshot::from_market_event(
                &tick.symbol,
                tick.bid,
                tick.ask,
                tick.last,
            );
        }
    }
}

#[async_trait]
impl<S, const N: usize> EventHandler<MarketEvent> for StrategyRunner<S, N>
where
    S: StrategyExecutor<N> + Send + Sync + 'static,
{
    async fn handle(&self, _event: MarketEvent) -> Result<(), EventBusError> {
        // We need mutable access to the strategy and snapshots.
        // Since EventHandler::handle takes &self, we would typically need internal mutability (Mutex/RwLock).
        // However, for this implementation, let's assume the StrategyRunner is wrapped in an Arc<Mutex<...>> 
        // or we change the design to use a channel receiver loop instead of EventHandler trait if mutability is hard.
        
        // Actually, the EventHandler trait is designed for shared access. 
        // If we need state, we should use interior mutability.
        // Let's wrap the mutable parts in a Mutex.
        // But wait, StrategyRunner definition above doesn't have Mutex.
        // I should probably redesign this to be a struct that holds an Arc<Mutex<State>>.
        
        // Let's do that refactor in a moment. For now, I'll implement the logic assuming I can get mutability,
        // which means I need to change the struct definition.
        
        Err(EventBusError::Upstream("StrategyRunner requires interior mutability refactor".into()))
    }
}

/// Thread-safe wrapper for StrategyRunner state
struct RunnerState<S, const N: usize> {
    strategy: S,
    snapshots: [MarketSnapshot; N],
    initialized: bool,
}

/// Thread-safe Strategy Runner suitable for EventDispatcher
pub struct ThreadSafeStrategyRunner<S, const N: usize> {
    state: Mutex<RunnerState<S, N>>,
    signal_sender: EventSender<SignalEvent>,
    account_id: AccountId,
    strategy_id: Uuid,
}

impl<S, const N: usize> ThreadSafeStrategyRunner<S, N>
where
    S: StrategyExecutor<N> + Send + Sync,
{
    pub fn new(
        strategy: S,
        signal_sender: EventSender<SignalEvent>,
        account_id: AccountId,
    ) -> Self {
        let snapshots = std::array::from_fn(|_| MarketSnapshot {
            symbol: String::new(),
            bid: rust_decimal::Decimal::ZERO,
            ask: rust_decimal::Decimal::ZERO,
            last: rust_decimal::Decimal::ZERO,
            timestamp: Utc::now(),
        });

        Self {
            state: Mutex::new(RunnerState {
                strategy,
                snapshots,
                initialized: false,
            }),
            signal_sender,
            account_id,
            strategy_id: Uuid::new_v4(),
        }
    }
}

#[async_trait]
impl<S, const N: usize> EventHandler<MarketEvent> for ThreadSafeStrategyRunner<S, N>
where
    S: StrategyExecutor<N> + Send + Sync + 'static,
{
    async fn handle(&self, event: MarketEvent) -> Result<(), EventBusError> {
        // Lock the state
        // Note: std::sync::Mutex blocks the thread. For async context, we should ideally use tokio::sync::Mutex.
        // But since we are in a sync context (EventHandler::handle is async but we can use blocking mutex if critical section is short),
        // let's stick to std::sync::Mutex for now as it's simpler and we added tokio dependency but maybe not full features.
        // Wait, I added tokio with "sync" feature. I should use tokio::sync::Mutex to be safe in async.
        // But I'll use std::sync::Mutex for now to match the imports I set up above.
        // Actually, holding a std::sync::Mutex across await points is bad.
        // But here we don't await inside the lock except maybe for publishing?
        // No, publishing is async if we use async send, but we use Try or Blocking.
        // Let's use std::sync::Mutex and be careful not to await inside.
        
        let mut state = self.state.lock().map_err(|_| EventBusError::Upstream("Mutex poisoned".into()))?;

        // Initialize if needed
        if !state.initialized {
            let init_ctx = StrategyInitContext {
                strategy_id: self.strategy_id,
                account_id: &self.account_id,
            };
            if let Err(e) = state.strategy.initialize(init_ctx) {
                error!("Failed to initialize strategy: {}", e);
                return Ok(()); // Don't crash the bus
            }
            state.initialized = true;
        }

        // Update snapshots
        if let MarketPayload::Tick { tick, .. } = event.payload() {
            for i in 0..N - 1 {
                state.snapshots[i] = state.snapshots[i + 1].clone();
            }
            state.snapshots[N - 1] = MarketSnapshot::from_market_event(
                &tick.symbol,
                tick.bid,
                tick.ask,
                tick.last,
            );
        }

        // Split borrows to avoid simultaneous mutable and immutable borrow of state
        let RunnerState { ref snapshots, ref mut strategy, .. } = *state;

        // Evaluate strategy
        let ctx = StrategyContext::new(
            &self.account_id,
            snapshots,
            Uuid::new_v4(),
            Utc::now(),
        ).with_events(std::slice::from_ref(&event));

        match strategy.evaluate(ctx) {
            Ok(decision) => {
                // Log output
                for log in decision.logs {
                    debug!("Strategy log: {}", log);
                }

                // Publish signals
                for signal_payload in decision.signals {
                    let metadata = event.metadata().child(
                        event_bus::EventSource::new("strategy_engine"),
                        signal_payload.priority,
                    );
                    
                    let signal_event = SignalEvent::new(metadata, signal_payload);
                    
                    // Use Try publish to avoid blocking
                    if let Err(e) = self.signal_sender.publish(signal_event, PublishMode::Try) {
                        error!("Failed to publish strategy signal: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Strategy evaluation failed: {}", e);
            }
        }

        Ok(())
    }
}
