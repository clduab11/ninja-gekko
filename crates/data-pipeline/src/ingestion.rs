use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use exchange_connectors::{ExchangeConnector, ExchangeId, StreamMessage};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, info};

/// Raw market message emitted by the ingestion stage.
pub type RawMarketMessage = (ExchangeId, StreamMessage);

/// Configuration for a single WebSocket ingestion task.
#[derive(Clone)]
pub struct IngestionConfig {
    /// Exchange identifier used for metadata tagging.
    pub exchange_id: ExchangeId,
    /// The upstream exchange connector. The trait contract remains unchanged.
    pub connector: Arc<dyn ExchangeConnector>,
    /// Symbols (or instrument identifiers) to subscribe to.
    pub symbols: Vec<String>,
    /// Optional heartbeat interval to emit synthetic keep-alive messages.
    pub heartbeat: Option<Duration>,
}

impl IngestionConfig {
    pub fn new(
        exchange_id: ExchangeId,
        connector: Arc<dyn ExchangeConnector>,
        symbols: Vec<String>,
    ) -> Self {
        Self {
            exchange_id,
            connector,
            symbols,
            heartbeat: None,
        }
    }

    pub fn with_heartbeat(mut self, interval: Duration) -> Self {
        self.heartbeat = Some(interval);
        self
    }
}

/// Error returned when an ingestion task cannot be spawned.
#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("exchange stream unavailable: {0}")]
    StreamUnavailable(String),
    #[error("websocket task join error: {0}")]
    Join(String),
}

/// Handle returned by the ingestion layer. Dropping the sender causes the task to exit.
pub struct IngestionHandle {
    join: JoinHandle<()>,
}

impl IngestionHandle {
    /// Awaits task completion, propagating panics as errors.
    pub async fn join(self) -> Result<(), IngestionError> {
        self.join
            .await
            .map_err(|err| IngestionError::Join(err.to_string()))
    }
}

/// Ingestor responsible for streaming WebSocket messages into crossbeam channels.
#[derive(Clone)]
pub struct StreamIngestor {
    config: IngestionConfig,
}

impl StreamIngestor {
    pub fn new(config: IngestionConfig) -> Self {
        Self { config }
    }

    /// Starts streaming messages from the underlying connector into the supplied sender.
    pub async fn spawn(
        self,
        outbound: Sender<RawMarketMessage>,
    ) -> Result<IngestionHandle, IngestionError> {
        let rx = self
            .config
            .connector
            .start_market_stream(self.config.symbols.clone())
            .await
            .map_err(|err| IngestionError::StreamUnavailable(err.to_string()))?;

        let exchange_id = self.config.exchange_id;
        let heartbeat_interval = self.config.heartbeat;
        let join = tokio::spawn(async move {
            ingest_loop(exchange_id, rx, outbound, heartbeat_interval).await;
        });

        Ok(IngestionHandle { join })
    }
}

async fn ingest_loop(
    exchange: ExchangeId,
    receiver: UnboundedReceiver<StreamMessage>,
    outbound: Sender<RawMarketMessage>,
    heartbeat: Option<Duration>,
) {
    let mut stream = UnboundedReceiverStream::new(receiver);
    let mut last_heartbeat = Instant::now();
    let heartbeat_interval = heartbeat.unwrap_or_else(|| Duration::from_secs(0));

    while let Some(message) = stream.next().await {
        if outbound.send((exchange, message)).is_err() {
            debug!(
                "{} ingestion shutting down: downstream closed",
                exchange_name(exchange)
            );
            return;
        }
        if heartbeat_interval.is_zero() {
            continue;
        }
        if last_heartbeat.elapsed() >= heartbeat_interval {
            if outbound.send((exchange, StreamMessage::Ping)).is_err() {
                return;
            }
            last_heartbeat = Instant::now();
        }
    }

    info!("{} WebSocket stream ended", exchange_name(exchange));
}

fn exchange_name(id: ExchangeId) -> &'static str {
    match id {
        ExchangeId::Kraken => "kraken",
        ExchangeId::Mock => "mock",
        ExchangeId::BinanceUs => "binance_us",
        ExchangeId::Oanda => "oanda",
    }
}
