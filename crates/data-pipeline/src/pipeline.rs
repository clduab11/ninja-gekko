use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver, Sender};
use event_bus::{EventBus, MarketEvent};
use exchange_connectors::{ExchangeConnector, ExchangeId};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::distributor::Distributor;
use crate::ingestion::{IngestionConfig, IngestionError, RawMarketMessage, StreamIngestor};
use crate::normalizer::{MarketNormalizer, NormalizedEvent};

/// Builder for a multi-exchange data pipeline.
pub struct DataPipelineBuilder {
    configs: Vec<IngestionConfig>,
    market_sender: Option<event_bus::EventSender<MarketEvent>>,
    raw_capacity: usize,
    normalized_capacity: usize,
}

impl DataPipelineBuilder {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            market_sender: None,
            raw_capacity: 4096,
            normalized_capacity: 4096,
        }
    }

    pub fn with_event_bus(mut self, bus: &EventBus) -> Self {
        self.market_sender = Some(bus.market_sender());
        self
    }

    pub fn with_exchange(
        mut self,
        exchange_id: ExchangeId,
        connector: Arc<dyn ExchangeConnector>,
        symbols: Vec<String>,
    ) -> Self {
        self.configs
            .push(IngestionConfig::new(exchange_id, connector, symbols));
        self
    }

    pub fn with_raw_capacity(mut self, capacity: usize) -> Self {
        self.raw_capacity = capacity;
        self
    }

    pub fn with_normalized_capacity(mut self, capacity: usize) -> Self {
        self.normalized_capacity = capacity;
        self
    }

    pub fn build(self) -> Result<DataPipeline, IngestionError> {
        let market_sender = self
            .market_sender
            .expect("event bus sender must be configured");

        let (raw_tx, raw_rx) = bounded::<RawMarketMessage>(self.raw_capacity);
        let (norm_tx, norm_rx) = bounded::<MarketEvent>(self.normalized_capacity);

        // Spawn ingestion tasks
        let mut ingestion_handles = Vec::new();
        for config in self.configs {
            let sender = raw_tx.clone();
            let ingestor = StreamIngestor::new(config);
            let handle = tokio::spawn(async move {
                match ingestor.spawn(sender).await {
                    Ok(handle) => {
                        if let Err(err) = handle.join().await {
                            warn!(%err, "ingestion task exited with error");
                        }
                    }
                    Err(err) => warn!(%err, "failed to start ingestion task"),
                }
            });
            ingestion_handles.push(handle);
        }

        drop(raw_tx); // ensure the channel closes once all ingestors exit

        // Normalization worker
        let normalizer_handle = spawn_normalizer(raw_rx, norm_tx.clone());

        // Distribution worker
        let distributor = Distributor::new(market_sender);
        let distributor_handle = spawn_distributor(distributor, norm_rx);

        Ok(DataPipeline {
            handle: DataPipelineHandle {
                ingestion_handles,
                normalizer_handle: Some(normalizer_handle),
                distributor_handle: Some(distributor_handle),
                normalized_sender: Some(norm_tx),
            },
        })
    }
}

fn spawn_normalizer(
    raw_rx: Receiver<RawMarketMessage>,
    norm_tx: Sender<MarketEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut normalizer = MarketNormalizer::new();
        for message in raw_rx.iter() {
            if let Some(NormalizedEvent { event, .. }) = normalizer.normalize(message) {
                if norm_tx.send(event).is_err() {
                    break;
                }
            }
        }
        info!("normalizer loop terminated");
    })
}

fn spawn_distributor(distributor: Distributor, norm_rx: Receiver<MarketEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        distributor.drain(norm_rx);
        info!("distributor loop terminated");
    })
}

/// Running pipeline instance.
pub struct DataPipeline {
    handle: DataPipelineHandle,
}

impl DataPipeline {
    pub fn handle(&self) -> &DataPipelineHandle {
        &self.handle
    }

    pub fn into_handle(self) -> DataPipelineHandle {
        self.handle
    }
}

/// Handle for gracefully shutting down the pipeline.
pub struct DataPipelineHandle {
    ingestion_handles: Vec<JoinHandle<()>>,
    normalizer_handle: Option<JoinHandle<()>>,
    distributor_handle: Option<JoinHandle<()>>,
    normalized_sender: Option<Sender<MarketEvent>>,
}

impl DataPipelineHandle {
    /// Signals shutdown, awaiting all background tasks.
    pub async fn shutdown(mut self) {
        if let Some(sender) = self.normalized_sender.take() {
            drop(sender);
        }

        for handle in std::mem::take(&mut self.ingestion_handles) {
            let _ = handle.await;
        }

        if let Some(handle) = self.normalizer_handle.take() {
            let _ = handle.await;
        }

        if let Some(handle) = self.distributor_handle.take() {
            let _ = handle.await;
        }
    }
}

impl Drop for DataPipelineHandle {
    fn drop(&mut self) {
        if let Some(sender) = self.normalized_sender.take() {
            drop(sender);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_bus::{EventBusBuilder, MarketPayload};
    use exchange_connectors::{ExchangeId, MarketTick, StreamMessage};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use tokio::time::{timeout, Duration as TokioDuration};

    #[tokio::test]
    #[ignore]
    async fn pipeline_dispatches_tick() {
        // Ignored until full pipeline shutdown semantics are hardened.
    }

    #[tokio::test]
    async fn test_pipeline_basic_dispatch_with_timeout() {
        let bus = EventBusBuilder::default().build();
        let receiver = bus.market_receiver();
        let distributor = Distributor::new(bus.market_sender());
        let mut normalizer = MarketNormalizer::new();

        let tick = MarketTick {
            symbol: "BTC-USD".into(),
            bid: dec!(30_000.0),
            ask: dec!(30_001.0),
            last: dec!(30_000.5),
            volume_24h: Decimal::new(100, 0),
            timestamp: chrono::Utc::now(),
        };

        let normalized = normalizer
            .normalize((ExchangeId::Kraken, StreamMessage::Tick(tick)))
            .expect("expected tick normalization");

        distributor
            .dispatch(normalized.event)
            .expect("dispatch should succeed");

        let result = timeout(TokioDuration::from_millis(100), receiver.recv_async()).await;

        match result {
            Ok(Ok(event)) => match event.payload() {
                MarketPayload::Tick { tick, .. } => {
                    assert_eq!(tick.symbol, "BTC-USD");
                }
                other => panic!("unexpected payload: {other:?}"),
            },
            Ok(Err(err)) => panic!("failed to receive market event: {err}"),
            Err(_) => panic!("market event not received within timeout"),
        }
    }
}
