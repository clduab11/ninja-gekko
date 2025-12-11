#![allow(clippy::ref_option_ref)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use event_bus::core_bridges::{OrderExecutionBridge, RiskLoggingHandler, SignalToOrderBridge};
use event_bus::{
    EventBusBuilder, EventDispatcherBuilder, EventHandler, EventMetadata, EventSender, EventSource,
    ExecutionEvent, MarketEvent, MarketPayload, Priority, PublishMode, SignalEvent,
    SignalEventPayload, StrategySignal,
};
use exchange_connectors::{
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, MarketTick, OrderSide,
    OrderStatus, OrderType as ExchangeOrderType, StreamMessage, TradingPair, TransferRequest,
    TransferStatus,
};
use ninja_gekko_core::order_manager::{DefaultFeeCalculator, DefaultRiskValidator, OrderManager};
use ninja_gekko_core::types::{OrderType, Portfolio};
use rust_decimal::Decimal;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tracing::info;
use uuid::Uuid;

struct MarketToSignalHandler {
    signal_sender: EventSender<SignalEvent>,
    account_id: String,
    order_quantity: Decimal,
    limit_price: Decimal,
}

#[async_trait]
impl EventHandler<MarketEvent> for MarketToSignalHandler {
    async fn handle(&self, event: MarketEvent) -> Result<(), event_bus::EventBusError> {
        let metadata = event
            .metadata()
            .child(EventSource::new("integration.strategy"), Priority::High);

        let symbol = match event.payload() {
            MarketPayload::Tick { tick, .. } => tick.symbol.clone(),
            MarketPayload::OrderBookSnapshot { pair, .. }
            | MarketPayload::OrderBookDelta { pair, .. } => pair.symbol.clone(),
        };

        let payload = SignalEventPayload {
            strategy_id: Uuid::new_v4(),
            account_id: self.account_id.clone(),
            priority: Priority::High,
            signal: StrategySignal {
                exchange: Some(ExchangeId::Kraken),
                symbol,
                side: ninja_gekko_core::types::OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: self.order_quantity,
                limit_price: Some(self.limit_price),
                confidence: 0.99,
                metadata: Default::default(),
            },
        };

        let signal_event = SignalEvent::new(metadata, payload);
        self.signal_sender
            .publish(signal_event, PublishMode::Blocking)?;
        Ok(())
    }
}

struct ExecutionRecorder {
    portfolio: Arc<RwLock<Portfolio>>,
    latency_tx: mpsc::Sender<Instant>,
}

#[async_trait]
impl EventHandler<ExecutionEvent> for ExecutionRecorder {
    async fn handle(&self, event: ExecutionEvent) -> Result<(), event_bus::EventBusError> {
        {
            let mut portfolio = self.portfolio.write().await;
            portfolio.update_from_execution(event.execution());
        }
        let _ = self.latency_tx.send(Instant::now()).await;
        Ok(())
    }
}

#[derive(Default)]
struct MockExchange;

#[async_trait]
impl ExchangeConnector for MockExchange {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Kraken
    }

    async fn connect(&mut self) -> Result<(), ExchangeError> {
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), ExchangeError> {
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        true
    }

    async fn get_trading_pairs(&self) -> Result<Vec<TradingPair>, ExchangeError> {
        Ok(vec![TradingPair {
            base: "BTC".into(),
            quote: "USD".into(),
            symbol: "BTC-USD".into(),
        }])
    }

    async fn get_balances(&self) -> Result<Vec<Balance>, ExchangeError> {
        Ok(vec![])
    }

    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: ExchangeOrderType,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<ExchangeOrder, ExchangeError> {
        let now = chrono::Utc::now();
        Ok(ExchangeOrder {
            id: Uuid::new_v4().to_string(),
            exchange_id: self.exchange_id(),
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::Filled,
            timestamp: now,
            fills: vec![exchange_connectors::Fill {
                id: Uuid::new_v4().to_string(),
                order_id: Uuid::new_v4().to_string(),
                price: price.unwrap_or(Decimal::new(300000, 2)),
                quantity,
                fee: Decimal::new(5, 3),
                timestamp: now,
            }],
        })
    }

    async fn cancel_order(&self, _order_id: &str) -> Result<ExchangeOrder, ExchangeError> {
        Err(ExchangeError::InvalidRequest("cancel not supported".into()))
    }

    async fn get_order(&self, _order_id: &str) -> Result<ExchangeOrder, ExchangeError> {
        Err(ExchangeError::InvalidRequest(
            "get_order not supported".into(),
        ))
    }

    async fn get_market_data(&self, symbol: &str) -> Result<MarketTick, ExchangeError> {
        Ok(MarketTick {
            symbol: symbol.to_string(),
            bid: Decimal::new(299_500, 2),
            ask: Decimal::new(300_500, 2),
            last: Decimal::new(300_000, 2),
            volume_24h: Decimal::new(1_500_000, 0),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn start_market_stream(
        &self,
        _symbols: Vec<String>,
    ) -> Result<mpsc::UnboundedReceiver<StreamMessage>, ExchangeError> {
        Err(ExchangeError::InvalidRequest("stream not supported".into()))
    }

    async fn start_order_stream(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<StreamMessage>, ExchangeError> {
        Err(ExchangeError::InvalidRequest(
            "order stream not supported".into(),
        ))
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> Result<String, ExchangeError> {
        Err(ExchangeError::InvalidRequest(
            "transfer not supported".into(),
        ))
    }

    async fn get_transfer_status(
        &self,
        _transfer_id: &str,
    ) -> Result<TransferStatus, ExchangeError> {
        Err(ExchangeError::InvalidRequest(
            "transfer status not supported".into(),
        ))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "pending end-to-end integration stabilization"]
async fn test_market_to_execution_pipeline() {
    let _ = tracing_subscriber::fmt::try_init();

    let bus = EventBusBuilder::default().build();
    let market_sender = bus.market_sender();
    let signal_sender = bus.signal_sender();
    let order_sender = bus.order_sender();
    let execution_sender = bus.execution_sender();

    let risk_manager = Box::new(DefaultRiskValidator::new(
        Decimal::new(1_000_000, 0),
        Decimal::new(5_000_000, 0),
        Decimal::new(10_000_000, 0),
    ));
    let fee_calculator = Box::new(DefaultFeeCalculator::new(Decimal::ZERO, Decimal::new(1, 3)));
    let order_manager = Arc::new(OrderManager::new(risk_manager, fee_calculator));
    let portfolio = Arc::new(RwLock::new(Portfolio::new("test-account".into())));

    let (latency_tx, mut latency_rx) = mpsc::channel::<Instant>(64);

    let market_handler = Arc::new(MarketToSignalHandler {
        signal_sender: signal_sender.clone(),
        account_id: "test-account".into(),
        order_quantity: Decimal::new(1, 0),
        limit_price: Decimal::new(300_000, 2),
    });

    let signal_bridge = Arc::new(SignalToOrderBridge::new(
        order_manager.clone(),
        order_sender,
        PublishMode::Blocking,
    ));

    let exchange: Arc<dyn ExchangeConnector> = Arc::new(MockExchange::default());
    let execution_bridge = Arc::new(OrderExecutionBridge::new(
        exchange,
        execution_sender,
        PublishMode::Blocking,
    ));

    let execution_handler = Arc::new(ExecutionRecorder {
        portfolio: portfolio.clone(),
        latency_tx,
    });

    let risk_handler = Arc::new(RiskLoggingHandler::new("event_bus.integration.risk"));

    let dispatcher = EventDispatcherBuilder::new(&bus)
        .on_market(market_handler)
        .on_signal(signal_bridge)
        .on_order(execution_bridge)
        .on_execution(execution_handler)
        .on_risk(risk_handler)
        .build();

    let controller = dispatcher.controller();
    let dispatcher_task = tokio::spawn(async move {
        dispatcher.run().await.expect("dispatcher run");
    });

    let tick = MarketTick {
        symbol: "BTC-USD".into(),
        bid: Decimal::new(299_500, 2),
        ask: Decimal::new(300_500, 2),
        last: Decimal::new(300_000, 2),
        volume_24h: Decimal::new(1_500_000, 0),
        timestamp: chrono::Utc::now(),
    };

    let metadata = EventMetadata::new(EventSource::new("market.synthetic"), Priority::High);
    let market_event = MarketEvent::new(
        metadata,
        MarketPayload::Tick {
            tick,
            pair: TradingPair {
                base: "BTC".into(),
                quote: "USD".into(),
                symbol: "BTC-USD".into(),
            },
        },
    );

    const ITERATIONS: usize = 32;
    let mut latencies = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let start = Instant::now();
        market_sender
            .publish(market_event.clone(), PublishMode::Blocking)
            .expect("publish market event");

        let completed = timeout(Duration::from_millis(10), latency_rx.recv())
            .await
            .expect("latency result")
            .expect("latency timestamp");
        latencies.push(completed.duration_since(start));
    }

    controller.shutdown();
    dispatcher_task.await.expect("dispatcher join");

    let max_latency = latencies
        .iter()
        .copied()
        .max()
        .unwrap_or(Duration::from_millis(0));
    info!(?max_latency, "measured pipeline latency");

    assert!(
        max_latency <= Duration::from_millis(5),
        "pipeline latency {:?} exceeded budget",
        max_latency
    );

    let snapshot = portfolio.read().await.clone();
    let position = snapshot.positions.get("BTC-USD").expect("position exists");
    assert_eq!(position.quantity, Decimal::new(ITERATIONS as i64, 0));
}
