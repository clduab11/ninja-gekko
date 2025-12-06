use anyhow::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{error, info};

pub struct McpExecutionClient {
    process: Option<Child>,
}

impl McpExecutionClient {
    pub fn new() -> Self {
        Self { process: None }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "-p", "mcp-server-trade", "--", "--dry-run=true"])
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::inherit()); 

        let child = cmd.spawn()?;
        self.process = Some(child);
        info!("MCP Execution Client started");
        Ok(())
    }

    pub async fn place_order(&mut self, symbol: &str, side: &str, quantity: Decimal) -> Result<String> {
        let req = json!({
            "jsonrpc": "2.0",
            "method": "place_order",
            "params": {
                "exchange": "Coinbase", // Hardcoded for now, should come from args
                "symbol": symbol,
                "side": side,
                "type": "Market",
                "quantity": quantity
            },
            "id": 1
        });

        self.send_request(req).await
    }

    async fn send_request(&mut self, req: serde_json::Value) -> Result<String> {
        if let Some(child) = &mut self.process {
            let stdin = child.stdin.as_mut().ok_or(anyhow::anyhow!("No stdin"))?;
            let req_str = serde_json::to_string(&req)?;
            stdin.write_all(req_str.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;

            let stdout = child.stdout.as_mut().ok_or(anyhow::anyhow!("No stdout"))?;
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            
            Ok(line)
        } else {
            Err(anyhow::anyhow!("Server not started"))
        }
    }
}
