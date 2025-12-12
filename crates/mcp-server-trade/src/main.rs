use anyhow::{Context, Result};
use clap::Parser;
use exchange_connectors::{ExchangeId, OrderSide, OrderType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead};
use std::str::FromStr;
use tracing::{debug, error, info};

mod safety;
use safety::SafetyValidator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    dry_run: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr to avoid corrupting stdout JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    info!("Starting MCP Trade Server (dry_run={})", args.dry_run);

    let safety = SafetyValidator::new(
        rust_decimal::Decimal::from(1000), // Default $1000 max size
        rust_decimal::Decimal::from(500),  // Default $500 daily loss
        args.dry_run,
    );

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        // Handle basic JSON-RPC
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => {
                let response = handle_request(&req, &safety).await;
                let response_str = serde_json::to_string(&response)?;
                println!("{}", response_str);
            }
            Err(e) => {
                error!("Failed to parse request: {}", e);
                // Send parse error
            }
        }
    }

    Ok(())
}

async fn handle_request(req: &JsonRpcRequest, safety: &SafetyValidator) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "tools/list" => Ok(json!({
            "tools": [
                {
                    "name": "place_order",
                    "description": "Place a new order on an exchange",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "exchange": { "type": "string", "enum": ["BinanceUs", "Oanda", "Kraken"] },
                            "symbol": { "type": "string" },
                            "side": { "type": "string", "enum": ["Buy", "Sell"] },
                            "type": { "type": "string", "enum": ["Market", "Limit"] },
                            "quantity": { "type": "number" },
                            "price": { "type": "number" }
                        },
                        "required": ["exchange", "symbol", "side", "type", "quantity"]
                    }
                },
                {
                    "name": "get_balance",
                    "description": "Get account balance for an exchange",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "exchange": { "type": "string", "enum": ["BinanceUs", "Oanda", "Kraken"] }
                        },
                        "required": ["exchange"]
                    }
                }
            ]
        })),
        "place_order" => handle_place_order(req.params.as_ref(), safety).await,
        "get_balance" => handle_get_balance(req.params.as_ref()).await,
        _ => Err(anyhow::anyhow!("Method not found")),
    };

    match result {
        Ok(val) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(val),
            error: None,
            id: req.id.clone(),
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
            id: req.id.clone(),
        },
    }
}

async fn handle_place_order(params: Option<&Value>, safety: &SafetyValidator) -> Result<Value> {
    let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;

    let exchange_str = params["exchange"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing exchange"))?;
    let symbol = params["symbol"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing symbol"))?;
    let qty_str = params["quantity"].to_string(); // Simple handling for now
    let qty = rust_decimal::Decimal::from_str_exact(&qty_str).unwrap_or_default();

    // Check safety
    safety.check_trade(symbol, qty, qty * rust_decimal::Decimal::from(100))?;

    if safety.is_dry_run() {
        info!(
            "Would place order: {} {} {} on {}",
            params["side"], qty, symbol, exchange_str
        );
        return Ok(json!({
            "status": "DryRun",
            "order_id": "dry_run_id_123",
            "message": "Order validated but not executed (dry_run=true)"
        }));
    }

    // Real execution would go here using exchange-connectors
    // let connector = get_connector(exchange_str)?;
    // connector.place_order(...).await?;

    Ok(json!({
        "status": "Filled",
        "order_id": "real_id_456"
    }))
}

async fn handle_get_balance(params: Option<&Value>) -> Result<Value> {
    // Placeholder
    Ok(json!({
        "available": "10000.00",
        "currency": "USD"
    }))
}
