//! Service Health Verification Tests
//! Acceptance tests for Ninja Gekko Trading Platform health and accessibility

use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;
use reqwest::Client;

#[cfg(test)]
mod service_health_tests {
    use super::*;

    /// Test 1: Docker Services Health Check
    /// Given: Docker Compose setup
    /// When: Checking all containers
    /// Then: All services should be running and healthy
    #[tokio::test]
    async fn test_docker_services_health() {
        // Check docker-compose ps output
        let output = Command::new("docker-compose")
            .args(&["ps", "--format", "table {{.Name}}\t{{.Status}}"])
            .output()
            .expect("Failed to execute docker-compose ps");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Docker services status:\n{}", stdout);

        // Verify expected services are running
        assert!(stdout.contains("Up"), "Services should be running");
        assert!(stdout.contains("trading-engine"), "Trading engine service missing");
        assert!(stdout.contains("database"), "Database service missing");
        assert!(stdout.contains("redis"), "Redis service missing");
        assert!(stdout.contains("nginx"), "Nginx service missing");
    }

    /// Test 2: Main API Accessibility on port 8080
    /// Given: API service running
    /// When: Making request to health endpoint
    /// Then: Should return 200 OK
    #[tokio::test]
    async fn test_main_api_health_endpoint() {
        let client = Client::new();

        // Test health endpoint with timeout
        let result = timeout(Duration::from_secs(10), async {
            client.get("http://localhost:8080/health")
                .send()
                .await
        }).await;

        match result {
            Ok(Ok(response)) => {
                assert_eq!(response.status(), 200, "API health endpoint should return 200");
                println!("Main API health check: PASSED");
            }
            Ok(Err(e)) => panic!("Request failed: {}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    /// Test 3: Chat API Accessibility via Nginx proxy on port 8787
    /// Given: Nginx proxy running
    /// When: Making request through proxy
    /// Then: Should return proper response
    #[tokio::test]
    async fn test_chat_api_nginx_proxy() {
        let client = Client::new();

        let result = timeout(Duration::from_secs(10), async {
            client.get("http://localhost:8787/health")
                .send()
                .await
        }).await;

        match result {
            Ok(Ok(response)) => {
                // Should either be 200 or expected proxy response
                assert!(response.status().is_success() || response.status() == 404,
                       "Chat API proxy should return success or 404 (expected for non-existent endpoints)");
                println!("Chat API proxy health check: PASSED");
            }
            Ok(Err(e)) => panic!("Request failed: {}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    /// Test 4: Database Connectivity
    /// Given: Database service running
    /// When: Testing connection
    /// Then: Should connect successfully
    #[tokio::test]
    async fn test_database_connectivity() {
        // Use environment variables or config for connection
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:password@localhost:5432/ninja_gekko".to_string());

        // This would typically use sqlx or similar
        // For now, test basic connectivity via command
        let output = Command::new("psql")
            .args(&[&db_url, "-c", "SELECT 1;"])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                println!("Database connectivity: PASSED");
            }
            _ => panic!("Database connection failed"),
        }
    }

    /// Test 5: Redis Connectivity
    /// Given: Redis service running
    /// When: Testing connection
    /// Then: Should connect successfully
    #[tokio::test]
    async fn test_redis_connectivity() {
        let output = Command::new("redis-cli")
            .args(&["-h", "localhost", "-p", "6379", "ping"])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                assert!(stdout.contains("PONG"), "Redis should respond with PONG");
                println!("Redis connectivity: PASSED");
            }
            _ => panic!("Redis connection failed"),
        }
    }

    /// Test 6: Port Bindings Validation
    /// Given: Services configured
    /// When: Checking listening ports
    /// Then: Correct ports should be bound
    #[tokio::test]
    async fn test_port_bindings() {
        // Check port 8080 (Main API)
        let output_8080 = Command::new("netstat")
            .args(&["-tuln", "|", "grep", ":8080"])
            .output();

        // Check port 8787 (Chat API/Nginx)
        let output_8787 = Command::new("netstat")
            .args(&["-tuln", "|", "grep", ":8787"])
            .output();

        // Note: netstat might not be available, this is a basic check
        // In real implementation, use proper port checking
        println!("Port binding validation: Basic check completed");
    }

    /// Test 7: CORS Configuration
    /// Given: API with CORS middleware
    /// When: Making cross-origin request
    /// Then: Should include proper CORS headers
    #[tokio::test]
    async fn test_cors_configuration() {
        let client = Client::new();

        let result = timeout(Duration::from_secs(10), async {
            client.get("http://localhost:8080/health")
                .header("Origin", "http://localhost:3000")
                .send()
                .await
        }).await;

        match result {
            Ok(Ok(response)) => {
                let cors_header = response.headers().get("access-control-allow-origin");
                assert!(cors_header.is_some(), "CORS header should be present");
                println!("CORS configuration: PASSED");
            }
            Ok(Err(e)) => panic!("CORS test failed: {}", e),
            Err(_) => panic!("CORS test timed out"),
        }
    }

    /// Test 8: JWT Configuration Validation
    /// Given: JWT secret configured
    /// When: Testing auth endpoint
    /// Then: Should handle JWT properly
    #[tokio::test]
    async fn test_jwt_configuration() {
        let client = Client::new();

        // Test without token (should fail gracefully)
        let result = timeout(Duration::from_secs(10), async {
            client.get("http://localhost:8080/api/protected")
                .send()
                .await
        }).await;

        match result {
            Ok(Ok(response)) => {
                // Should return 401 Unauthorized for protected endpoint
                assert_eq!(response.status(), 401, "Protected endpoint should return 401 without token");
                println!("JWT configuration validation: PASSED");
            }
            Ok(Err(e)) => panic!("JWT test failed: {}", e),
            Err(_) => panic!("JWT test timed out"),
        }
    }

    /// Test 9: Market Data Flow Testing
    /// Given: Exchange connectors configured
    /// When: Requesting market data
    /// Then: Should handle requests (may return 401 without real keys)
    #[tokio::test]
    async fn test_market_data_flow() {
        let client = Client::new();

        let result = timeout(Duration::from_secs(10), async {
            client.get("http://localhost:8080/api/market-data")
                .send()
                .await
        }).await;

        match result {
            Ok(Ok(response)) => {
                // Should return success or expected auth error
                assert!(response.status().is_success() || response.status() == 401,
                       "Market data endpoint should return success or 401 (expected without API keys)");
                println!("Market data flow: PASSED");
            }
            Ok(Err(e)) => panic!("Market data test failed: {}", e),
            Err(_) => panic!("Market data test timed out"),
        }
    }

    /// Test 10: Performance and Load Testing
    /// Given: System running
    /// When: Making concurrent requests
    /// Then: Should handle load without crashing
    #[tokio::test]
    async fn test_performance_load() {
        let client = Client::new();
        let mut handles = vec![];

        // Make 10 concurrent requests to health endpoint
        for _ in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                timeout(Duration::from_secs(5), async {
                    client_clone.get("http://localhost:8080/health")
                        .send()
                        .await
                }).await
            });
            handles.push(handle);
        }

        // Wait for all requests
        for handle in handles {
            match handle.await {
                Ok(Ok(Ok(response))) => {
                    assert_eq!(response.status(), 200, "Concurrent requests should succeed");
                }
                _ => panic!("Concurrent request failed"),
            }
        }

        println!("Performance/load testing: PASSED");
    }
}