#![allow(missing_docs)]

use std::fmt;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use tokio::task;

#[cfg(feature = "exchange-integration")]
use crate::envelope::MarketEvent;
use crate::envelope::RiskEvent;
#[cfg(feature = "core-integration")]
use crate::envelope::{ExecutionEvent, OrderEvent, SignalEvent};
use crate::error::EventBusError;

/// Result alias for publishing events to the bus.
pub type EventPublishResult = Result<(), EventBusError>;

/// Publication mode controlling how to handle backpressure on bounded channels.
#[derive(Debug, Clone, Copy)]
pub enum PublishMode {
    /// Block until there is capacity.
    Blocking,
    /// Return immediately with an error if the channel is full.
    Try,
    /// Block until capacity is available or the timeout elapses.
    Timeout(Duration),
}

impl Default for PublishMode {
    fn default() -> Self {
        PublishMode::Blocking
    }
}

/// Sender wrapper that enforces publish semantics.
#[derive(Clone)]
pub struct EventSender<T: Send + 'static> {
    inner: Sender<T>,
}

impl<T: Send + 'static> EventSender<T> {
    fn new(inner: Sender<T>) -> Self {
        Self { inner }
    }

    /// Publishes an event according to the supplied mode.
    pub fn publish(&self, event: T, mode: PublishMode) -> EventPublishResult {
        match mode {
            PublishMode::Blocking => self
                .inner
                .send(event)
                .map_err(EventBusError::from_send_error),
            PublishMode::Try => self
                .inner
                .try_send(event)
                .map_err(EventBusError::from_try_send_error),
            PublishMode::Timeout(timeout) => self
                .inner
                .send_timeout(event, timeout)
                .map_err(|err| EventBusError::from_send_timeout_error(err, timeout)),
        }
    }

    /// Attempts to publish without blocking, returning whether it succeeded.
    pub fn try_publish(&self, event: T) -> EventPublishResult {
        self.publish(event, PublishMode::Try)
    }
}

impl<T: Send + 'static> fmt::Debug for EventSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventSender").finish_non_exhaustive()
    }
}

/// Receiver wrapper with async-friendly helpers.
#[derive(Clone)]
pub struct EventReceiver<T: Send + 'static> {
    inner: Receiver<T>,
}

impl<T: Send + 'static> EventReceiver<T> {
    fn new(inner: Receiver<T>) -> Self {
        Self { inner }
    }

    /// Receives synchronously, blocking the current thread.
    pub fn recv(&self) -> Result<T, EventBusError> {
        self.inner.recv().map_err(EventBusError::from_recv_error)
    }

    /// Receives synchronously with timeout semantics.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, EventBusError> {
        self.inner
            .recv_timeout(timeout)
            .map_err(EventBusError::from_recv_timeout)
    }

    /// Asynchronously awaits the next event, delegating to a blocking task so
    /// it plays nicely with Tokio's scheduler.
    pub async fn recv_async(&self) -> Result<T, EventBusError> {
        let rx = self.inner.clone();
        task::spawn_blocking(move || rx.recv())
            .await
            .map_err(|err| EventBusError::Join(err.to_string()))?
            .map_err(EventBusError::from_recv_error)
    }

    /// Attempts to receive without blocking.
    pub fn try_recv(&self) -> Result<T, EventBusError> {
        self.inner
            .try_recv()
            .map_err(EventBusError::from_try_recv_error)
    }
}

impl<T: Send + 'static> fmt::Debug for EventReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventReceiver").finish_non_exhaustive()
    }
}

/// Builder configuring channel capacities and timeouts.
#[derive(Debug, Clone)]
pub struct EventBusBuilder {
    market_capacity: usize,
    signal_capacity: usize,
    order_capacity: usize,
    execution_capacity: usize,
    risk_capacity: usize,
    publish_timeout: Duration,
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self {
            market_capacity: 4_096,
            signal_capacity: 2_048,
            order_capacity: 2_048,
            execution_capacity: 4_096,
            risk_capacity: 256,
            publish_timeout: Duration::from_millis(1),
        }
    }
}

impl EventBusBuilder {
    /// Adjusts the capacity for market events.
    pub fn market_capacity(mut self, capacity: usize) -> Self {
        self.market_capacity = capacity;
        self
    }

    /// Adjusts the capacity for signal events.
    pub fn signal_capacity(mut self, capacity: usize) -> Self {
        self.signal_capacity = capacity;
        self
    }

    /// Adjusts the capacity for order events.
    pub fn order_capacity(mut self, capacity: usize) -> Self {
        self.order_capacity = capacity;
        self
    }

    /// Adjusts the capacity for execution events.
    pub fn execution_capacity(mut self, capacity: usize) -> Self {
        self.execution_capacity = capacity;
        self
    }

    /// Adjusts the capacity for risk events.
    pub fn risk_capacity(mut self, capacity: usize) -> Self {
        self.risk_capacity = capacity;
        self
    }

    /// Sets the default publish timeout for blocking operations.
    pub fn publish_timeout(mut self, timeout: Duration) -> Self {
        self.publish_timeout = timeout;
        self
    }

    /// Builds the event bus, allocating bounded crossbeam channels per event kind.
    pub fn build(self) -> EventBus {
        EventBus::new(self)
    }
}

/// Central event bus exposing typed senders and receivers for the five core event streams.
#[derive(Debug, Clone)]
pub struct EventBus {
    #[cfg(feature = "exchange-integration")]
    market_tx: Sender<MarketEvent>,
    #[cfg(feature = "exchange-integration")]
    market_rx: Receiver<MarketEvent>,

    #[cfg(feature = "core-integration")]
    signal_tx: Sender<SignalEvent>,
    #[cfg(feature = "core-integration")]
    signal_rx: Receiver<SignalEvent>,

    #[cfg(feature = "core-integration")]
    order_tx: Sender<OrderEvent>,
    #[cfg(feature = "core-integration")]
    order_rx: Receiver<OrderEvent>,

    #[cfg(feature = "core-integration")]
    execution_tx: Sender<ExecutionEvent>,
    #[cfg(feature = "core-integration")]
    execution_rx: Receiver<ExecutionEvent>,

    risk_tx: Sender<RiskEvent>,
    risk_rx: Receiver<RiskEvent>,

    publish_timeout: Duration,
}

impl EventBus {
    fn new(builder: EventBusBuilder) -> Self {
        #[cfg(feature = "exchange-integration")]
        let (market_tx, market_rx) = bounded(builder.market_capacity);
        #[cfg(feature = "core-integration")]
        let (signal_tx, signal_rx) = bounded(builder.signal_capacity);
        #[cfg(feature = "core-integration")]
        let (order_tx, order_rx) = bounded(builder.order_capacity);
        #[cfg(feature = "core-integration")]
        let (execution_tx, execution_rx) = bounded(builder.execution_capacity);
        let (risk_tx, risk_rx) = bounded(builder.risk_capacity);

        Self {
            #[cfg(feature = "exchange-integration")]
            market_tx,
            #[cfg(feature = "exchange-integration")]
            market_rx,
            #[cfg(feature = "core-integration")]
            signal_tx,
            #[cfg(feature = "core-integration")]
            signal_rx,
            #[cfg(feature = "core-integration")]
            order_tx,
            #[cfg(feature = "core-integration")]
            order_rx,
            #[cfg(feature = "core-integration")]
            execution_tx,
            #[cfg(feature = "core-integration")]
            execution_rx,
            risk_tx,
            risk_rx,
            publish_timeout: builder.publish_timeout,
        }
    }

    /// Default publish timeout for blocking modes.
    pub fn publish_timeout(&self) -> Duration {
        self.publish_timeout
    }

    #[cfg(feature = "exchange-integration")]
    /// Returns a sender for market events.
    pub fn market_sender(&self) -> EventSender<MarketEvent> {
        EventSender::new(self.market_tx.clone())
    }

    #[cfg(feature = "exchange-integration")]
    /// Returns a receiver for market events.
    pub fn market_receiver(&self) -> EventReceiver<MarketEvent> {
        EventReceiver::new(self.market_rx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the sender for strategy signal events.
    pub fn signal_sender(&self) -> EventSender<SignalEvent> {
        EventSender::new(self.signal_tx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the receiver for strategy signal events.
    pub fn signal_receiver(&self) -> EventReceiver<SignalEvent> {
        EventReceiver::new(self.signal_rx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the sender for order events emitted by the portfolio stage.
    pub fn order_sender(&self) -> EventSender<OrderEvent> {
        EventSender::new(self.order_tx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the receiver for order events.
    pub fn order_receiver(&self) -> EventReceiver<OrderEvent> {
        EventReceiver::new(self.order_rx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the sender for execution events produced by exchange bridges.
    pub fn execution_sender(&self) -> EventSender<ExecutionEvent> {
        EventSender::new(self.execution_tx.clone())
    }

    #[cfg(feature = "core-integration")]
    /// Returns the receiver for execution events.
    pub fn execution_receiver(&self) -> EventReceiver<ExecutionEvent> {
        EventReceiver::new(self.execution_rx.clone())
    }

    /// Returns the sender for risk management events.
    pub fn risk_sender(&self) -> EventSender<RiskEvent> {
        EventSender::new(self.risk_tx.clone())
    }

    /// Returns the receiver for risk management events.
    pub fn risk_receiver(&self) -> EventReceiver<RiskEvent> {
        EventReceiver::new(self.risk_rx.clone())
    }
}
