//! OANDA v20 streaming connector focused on market data ingestion.

use crate::{
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, ExchangeResult,
    MarketTick, OrderSide, OrderType, StreamMessage, TransferRequest, TransferStatus,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use parking_lot::RwLock;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

const OANDA_STREAM_PRACTICE: &str = "https://stream-fxpractice.oanda.com/v3";
const OANDA_STREAM_LIVE: &str = "https://stream-fxtrade.oanda.com/v3";

#[derive(Clone)]
struct OandaCredentials {
    account_id: String,
    access_token: String,
}

struct OandaInner {
    client: Client,
    connected: AtomicBool,
    stream_host: String,
    credentials: RwLock<Option<OandaCredentials>>,
}

/// OANDA v20 connector offering streaming price data.
pub struct OandaConnector {
    inner: Arc<OandaInner>,
}

impl OandaConnector {
    pub fn new() -> Self {
        Self::new_with_host(OANDA_STREAM_PRACTICE)
    }

    pub fn with_credentials(
        account_id: impl Into<String>,
        access_token: impl Into<String>,
        practice: bool,
    ) -> Self {
        let connector = Self::new_with_host(if practice {
            OANDA_STREAM_PRACTICE
        } else {
            OANDA_STREAM_LIVE
        });
        connector.set_credentials(account_id, access_token);
        connector
    }

    pub fn set_credentials(&self, account_id: impl Into<String>, access_token: impl Into<String>) {
        let credentials = OandaCredentials {
            account_id: account_id.into(),
            access_token: access_token.into(),
        };
        *self.inner.credentials.write() = Some(credentials);
    }

    fn new_with_host(host: &str) -> Self {
        Self {
            inner: Arc::new(OandaInner {
                client: Client::new(),
                connected: AtomicBool::new(false),
                stream_host: host.to_string(),
                credentials: RwLock::new(None),
            }),
        }
    }
}

#[async_trait]
impl ExchangeConnector for OandaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Oanda
    }

    async fn connect(&mut self) -> ExchangeResult<()> {
        info!("connecting to OANDA streaming API");
        self.inner.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&mut self) -> ExchangeResult<()> {
        self.inner.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.inner.connected.load(Ordering::SeqCst)
    }

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<crate::TradingPair>> {
        // REST discovery is out of scope for this phase.
        Ok(vec![])
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        Ok(vec![])
    }

    async fn place_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _order_type: OrderType,
        _quantity: Decimal,
        _price: Option<Decimal>,
    ) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for OANDA connector".to_string(),
        ))
    }

    async fn cancel_order(&self, _order_id: &str) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for OANDA connector".to_string(),
        ))
    }

    async fn get_order(&self, _order_id: &str) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for OANDA connector".to_string(),
        ))
    }

    async fn get_market_data(&self, _symbol: &str) -> ExchangeResult<MarketTick> {
        Err(ExchangeError::InvalidRequest(
            "Use streaming market data for OANDA".to_string(),
        ))
    }

    async fn start_market_stream(
        &self,
        symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        if symbols.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "at least one instrument must be provided for OANDA streaming".into(),
            ));
        }

        let credentials = self.inner.credentials.read().clone().ok_or_else(|| {
            ExchangeError::InvalidRequest("OANDA credentials not configured".into())
        })?;

        let (tx, rx) = mpsc::unbounded_channel();
        let client = self.inner.client.clone();
        let host = self.inner.stream_host.clone();
        let instruments = symbols.join(",");

        tokio::spawn(async move {
            if let Err(err) =
                run_oanda_price_stream(client, host, credentials, instruments, tx.clone()).await
            {
                error!(%err, "oanda price stream terminated");
            }
        });

        Ok(rx)
    }

    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        let (_tx, rx) = mpsc::unbounded_channel();
        warn!("OANDA order streaming not available via public endpoints");
        Ok(rx)
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> ExchangeResult<String> {
        Err(ExchangeError::InvalidRequest(
            "Fund transfers not supported by the OANDA connector".to_string(),
        ))
    }

    async fn get_transfer_status(&self, _transfer_id: &str) -> ExchangeResult<TransferStatus> {
        Err(ExchangeError::InvalidRequest(
            "Fund transfers not supported by the OANDA connector".to_string(),
        ))
    }

    async fn get_candles(
        &self,
        _symbol: &str,
        _timeframe: crate::Timeframe,
        _start: Option<chrono::DateTime<chrono::Utc>>,
        _end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ExchangeResult<Vec<crate::Candle>> {
        Err(ExchangeError::InvalidRequest(
            "Historical candles not implemented for OANDA connector".to_string(),
        ))
    }
}

async fn run_oanda_price_stream(
    client: Client,
    host: String,
    credentials: OandaCredentials,
    instruments: String,
    sender: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let mut attempt: u32 = 0;
    loop {
        attempt = attempt.saturating_add(1);
        let url = format!(
            "{}/accounts/{}/pricing/stream",
            host, credentials.account_id
        );

        debug!(
            attempt,
            url, instruments, "connecting to OANDA pricing stream"
        );

        let response = client
            .get(&url)
            .query(&[("instruments", instruments.as_str())])
            .header("Accept-Datetime-Format", "RFC3339")
            .bearer_auth(&credentials.access_token)
            .send()
            .await
            .map_err(|err| ExchangeError::Network(err.to_string()));

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!(status = %resp.status(), "oanda streaming request failed");
                } else {
                    attempt = 0;
                    if let Err(err) = consume_oanda_stream(resp, &sender).await {
                        warn!(%err, "error while consuming OANDA stream");
                    }
                }
            }
            Err(err) => {
                warn!(%err, "failed to open OANDA stream");
            }
        }

        if sender.is_closed() {
            debug!("oanda subscriber dropped channel; terminating stream");
            return Ok(());
        }

        let delay = Duration::from_millis(
            (600.0 * 1.5_f64.powi(attempt.min(8) as i32)).min(8_000.0) as u64,
        );
        sleep(delay).await;
    }
}

async fn consume_oanda_stream(
    response: reqwest::Response,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|err| ExchangeError::Network(err.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim();
            if !line.is_empty() {
                if let Err(err) = handle_oanda_line(line, sender) {
                    warn!(%err, "failed to parse OANDA tick");
                }
            }
            buffer.drain(..=pos);
        }

        if sender.is_closed() {
            return Ok(());
        }
    }

    Ok(())
}

fn handle_oanda_line(
    line: &str,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|err| ExchangeError::Network(format!("invalid OANDA payload: {err}")))?;

    match value.get("type").and_then(|v| v.as_str()) {
        Some("PRICE") => emit_oanda_price(&value, sender)?,
        Some("HEARTBEAT") => debug!("oanda heartbeat received"),
        _ => {}
    }
    Ok(())
}

fn emit_oanda_price(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let instrument = value
        .get("instrument")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing instrument in OANDA price".into()))?;

    let bids = value
        .get("bids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ExchangeError::Network("missing bids in OANDA price".into()))?;
    let asks = value
        .get("asks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ExchangeError::Network("missing asks in OANDA price".into()))?;

    let bid_price = bids
        .get(0)
        .and_then(|entry| entry.get("price"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing bid price".into()))?;
    let ask_price = asks
        .get(0)
        .and_then(|entry| entry.get("price"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing ask price".into()))?;

    let bid = Decimal::from_str(bid_price)
        .map_err(|err| ExchangeError::Network(format!("invalid bid price: {err}")))?;
    let ask = Decimal::from_str(ask_price)
        .map_err(|err| ExchangeError::Network(format!("invalid ask price: {err}")))?;
    let last = (bid + ask) / Decimal::from(2u32);

    let timestamp = value
        .get("time")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let tick = MarketTick {
        symbol: instrument.replace('_', "-"),
        bid,
        ask,
        last,
        volume_24h: Decimal::ZERO,
        timestamp,
    };

    let _ = sender.send(StreamMessage::Tick(tick));
    Ok(())
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn instrument_formatting() {
        assert_eq!("EUR-USD", "EUR_USD".replace('_', "-"));
    }
}
