//! Integration tests using testcontainers for database connectivity

use testcontainers::{clients::Cli, images::postgres::Postgres, RunnableImage};

#[tokio::test]
async fn test_postgres_connection() {
    let docker = Cli::default();
    let postgres = Postgres::default();
    let node = docker.run(postgres);

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );

    let pool = sqlx::PgPool::connect(&connection_string).await.unwrap();

    // Test basic query
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(42_i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(row.0, 42);

    pool.close().await;
}

#[tokio::test]
async fn test_postgres_table_creation() {
    let docker = Cli::default();
    let postgres = Postgres::default();
    let node = docker.run(postgres);

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );

    let pool = sqlx::PgPool::connect(&connection_string).await.unwrap();

    // Create a test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS test_trades (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(12, 2) NOT NULL,
            quantity DECIMAL(12, 8) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert a test row
    sqlx::query("INSERT INTO test_trades (symbol, price, quantity) VALUES ($1, $2, $3)")
        .bind("BTC-USD")
        .bind(50000.00)
        .bind(0.5)
        .execute(&pool)
        .await
        .unwrap();

    // Verify insertion
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_trades")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count.0, 1);

    pool.close().await;
}
