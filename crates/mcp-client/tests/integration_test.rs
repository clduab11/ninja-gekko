use mcp_client::execution::McpExecutionClient;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::test]
async fn test_dry_run_execution() {
    // Only run if we can build the server
    // This assumes `cargo build -p mcp-server-trade` has been run or will be handled by the test runner
    
    let mut client = McpExecutionClient::new();
    
    // Start the server subprocess
    match client.start().await {
        Ok(_) => println!("Client started"),
        Err(e) => {
            eprintln!("Failed to start client (likely need to build mcp-server-trade first): {}", e);
            return;
        }
    }

    let qty = Decimal::from_str("0.1").unwrap();
    let response = client.place_order("BTC-USD", "Buy", qty).await;
    
    match response {
        Ok(res) => {
            println!("Response: {}", res);
            assert!(res.contains("DryRun"), "Response should indicate DryRun mode");
            assert!(res.contains("dry_run_id"), "Response should contain dry run ID");
        }
        Err(e) => panic!("Call failed: {}", e),
    }
}
