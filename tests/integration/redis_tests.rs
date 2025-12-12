//! Integration tests using testcontainers for Redis connectivity

use testcontainers::{clients::Cli, GenericImage};

#[tokio::test]
#[ignore = "requires docker"]
async fn test_redis_connection() {
    let docker = Cli::default();
    let redis_image = GenericImage::new("redis", "7").with_exposed_port(6379);
    let node = docker.run(redis_image);

    let host_port = node.get_host_port_ipv4(6379);
    let connection_string = format!("redis://127.0.0.1:{}", host_port);

    // Wait for redis to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client = redis::Client::open(connection_string.as_str()).unwrap();
    let mut con = client.get_multiplexed_tokio_connection().await.unwrap();

    // Test SET command
    redis::cmd("SET")
        .arg("test_key")
        .arg("test_value")
        .query_async::<_, ()>(&mut con)
        .await
        .unwrap();

    // Test GET command
    let value: String = redis::cmd("GET")
        .arg("test_key")
        .query_async(&mut con)
        .await
        .unwrap();

    assert_eq!(value, "test_value");
}

#[tokio::test]
#[ignore = "requires docker"]
async fn test_redis_hash_operations() {
    let docker = Cli::default();
    let redis_image = GenericImage::new("redis", "7").with_exposed_port(6379);
    let node = docker.run(redis_image);

    let host_port = node.get_host_port_ipv4(6379);
    let connection_string = format!("redis://127.0.0.1:{}", host_port);

    // Wait for redis to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client = redis::Client::open(connection_string.as_str()).unwrap();
    let mut con = client.get_multiplexed_tokio_connection().await.unwrap();

    // Test HSET command
    redis::cmd("HSET")
        .arg("portfolio:BTC")
        .arg("balance")
        .arg("1.5")
        .arg("value_usd")
        .arg("75000.00")
        .query_async::<_, ()>(&mut con)
        .await
        .unwrap();

    // Test HGET command
    let balance: String = redis::cmd("HGET")
        .arg("portfolio:BTC")
        .arg("balance")
        .query_async(&mut con)
        .await
        .unwrap();

    assert_eq!(balance, "1.5");

    // Test HGETALL command
    let all_fields: Vec<String> = redis::cmd("HGETALL")
        .arg("portfolio:BTC")
        .query_async(&mut con)
        .await
        .unwrap();

    assert!(all_fields.contains(&"balance".to_string()));
    assert!(all_fields.contains(&"1.5".to_string()));
}
