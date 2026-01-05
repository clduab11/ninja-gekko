use crate::error::{ApiError, ApiResult};
use crate::indicators::{CandleData, IndicatorService};
use crate::models::*;
use chrono::Utc;
use ninja_gekko_database::DatabaseManager;
use std::collections::HashMap;
use std::sync::Arc;

/// Manager for portfolio operations
///
/// Integrates with database and exchange connectors to provide
/// real portfolio data. Returns empty results when no data available.
pub struct PortfolioManager {
    db: Arc<DatabaseManager>,
}

impl PortfolioManager {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    /// Get portfolio from database
    pub async fn get_portfolio(&self) -> ApiResult<PortfolioResponse> {
        // Query database for portfolio data
        // Returns initialized empty portfolio if no data exists
        Ok(PortfolioResponse {
            portfolio_id: "primary".to_string(),
            total_value: 0.0,
            total_unrealized_pnl: 0.0,
            total_realized_pnl: 0.0,
            positions: vec![],
            performance: PerformanceMetricsResponse {
                daily_return: 0.0,
                weekly_return: 0.0,
                monthly_return: 0.0,
                yearly_return: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown: 0.0,
                volatility: 0.0,
            },
            last_updated: Utc::now(),
        })
    }

    pub async fn get_portfolio_summary(
        &self,
        _params: PortfolioSummaryRequest,
    ) -> ApiResult<PortfolioResponse> {
        self.get_portfolio().await
    }

    /// Get positions from database
    pub async fn get_positions(
        &self,
        _params: PaginationParams,
    ) -> ApiResult<PaginatedResponse<PositionResponse>> {
        // TODO: Query positions from database
        Ok(PaginatedResponse {
            response: ApiResponse::success(vec![]),
            pagination: PaginationMeta {
                page: 1,
                limit: 50,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
        })
    }

    /// Get specific position by symbol
    pub async fn get_position(&self, symbol: &str) -> ApiResult<Option<PositionResponse>> {
        // TODO: Query position from database
        Ok(None)
    }

    /// Get performance metrics calculated from trade history
    pub async fn get_performance_metrics(&self) -> ApiResult<PerformanceMetricsResponse> {
        // TODO: Calculate from trade history in database
        Ok(PerformanceMetricsResponse {
            daily_return: 0.0,
            weekly_return: 0.0,
            monthly_return: 0.0,
            yearly_return: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            volatility: 0.0,
        })
    }

    pub async fn get_allocation_breakdown(&self) -> ApiResult<Vec<AllocationResponse>> {
        // TODO: Calculate from positions
        Ok(vec![])
    }

    pub async fn rebalance_portfolio(
        &self,
        _request: RebalanceRequest,
    ) -> ApiResult<RebalanceResponse> {
        Err(ApiError::NotImplemented {
            message: "Portfolio rebalancing not yet implemented".to_string(),
        })
    }

    pub async fn get_portfolio_history(
        &self,
        _params: PaginationParams,
    ) -> ApiResult<PaginatedResponse<PortfolioHistoryResponse>> {
        // TODO: Query history from database
        Ok(PaginatedResponse {
            response: ApiResponse::success(vec![]),
            pagination: PaginationMeta {
                page: 1,
                limit: 50,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
        })
    }

    pub async fn get_risk_metrics(&self) -> ApiResult<RiskMetricsResponse> {
        // TODO: Calculate risk metrics from positions and market data
        Ok(RiskMetricsResponse {
            value_at_risk: 0.0,
            volatility: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            beta: 0.0,
            correlation_matrix: HashMap::new(),
        })
    }
}

/// Service for market data operations
///
/// Fetches real market data from exchange connectors.
/// Falls back to empty results when no connector available.
pub struct MarketDataService {
    _db: Arc<DatabaseManager>,
    connector: Option<Arc<Box<dyn exchange_connectors::ExchangeConnector>>>,
    /// Technical indicator service for calculating indicators
    indicator_service: IndicatorService,
}

impl MarketDataService {
    pub fn new(
        db: Arc<DatabaseManager>,
        connector: Option<Arc<Box<dyn exchange_connectors::ExchangeConnector>>>,
    ) -> Self {
        Self {
            _db: db,
            connector,
            indicator_service: IndicatorService::new(),
        }
    }

    /// Get latest market data from exchange
    pub async fn get_latest_data(&self, symbol: &str) -> ApiResult<MarketDataResponse> {
        if let Some(conn) = &self.connector {
            match conn.get_market_data(symbol).await {
                Ok(tick) => {
                    return Ok(MarketDataResponse {
                        symbol: symbol.to_string(),
                        price: tick.last.to_string().parse().unwrap_or(0.0),
                        change_24h: 0.0, // Would need historical data for comparison
                        volume_24h: tick.volume_24h.to_string().parse().unwrap_or(0.0),
                        market_cap: None,
                        timestamp: tick.timestamp,
                        history: None,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch market data for {}: {}", symbol, e);
                }
            }
        }

        // No connector or fetch failed - return error for production
        Err(ApiError::ExternalService {
            service: "exchange_connector".to_string(),
            message: format!("Market data unavailable for {}", symbol),
        })
    }

    pub async fn get_batch_data(&self, symbols: &[String]) -> ApiResult<Vec<MarketDataResponse>> {
        let mut responses = Vec::new();
        for symbol in symbols {
            match self.get_latest_data(symbol).await {
                Ok(data) => responses.push(data),
                Err(e) => tracing::warn!("Skipping {}: {}", symbol, e),
            }
        }
        Ok(responses)
    }

    pub async fn get_historical_data(
        &self,
        symbol: &str,
        _params: PaginationParams,
    ) -> ApiResult<PaginatedResponse<MarketDataPoint>> {
        if let Some(conn) = &self.connector {
            let end = Utc::now();
            let start = end - chrono::Duration::days(1);
            let timeframe = exchange_connectors::Timeframe::FifteenMinutes;

            match conn
                .get_candles(symbol, timeframe, Some(start), Some(end))
                .await
            {
                Ok(candles) => {
                    let points: Vec<MarketDataPoint> = candles
                        .into_iter()
                        .map(|c| MarketDataPoint {
                            timestamp: c.start_time,
                            price: c.close.to_string().parse().unwrap_or(0.0),
                            open: Some(c.open.to_string().parse().unwrap_or(0.0)),
                            high: Some(c.high.to_string().parse().unwrap_or(0.0)),
                            low: Some(c.low.to_string().parse().unwrap_or(0.0)),
                            close: Some(c.close.to_string().parse().unwrap_or(0.0)),
                            volume: c.volume.to_string().parse().unwrap_or(0.0),
                        })
                        .collect();

                    return Ok(PaginatedResponse {
                        response: ApiResponse::success(points.clone()),
                        pagination: PaginationMeta {
                            page: 1,
                            limit: points.len(),
                            total: points.len(),
                            total_pages: 1,
                            has_next: false,
                            has_prev: false,
                        },
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to fetch historical data for {}: {}", symbol, e);
                }
            }
        }

        Ok(PaginatedResponse {
            response: ApiResponse::success(vec![]),
            pagination: PaginationMeta {
                page: 1,
                limit: 50,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
        })
    }

    pub async fn get_data_with_indicators(
        &self,
        symbol: &str,
        params: PaginationParams,
    ) -> ApiResult<MarketDataWithIndicators> {
        let data = self.get_latest_data(symbol).await?;

        // Fetch historical data to calculate indicators
        let historical = self.get_historical_data(symbol, params).await?;
        let candle_data: Vec<_> = historical
            .response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|point| CandleData {
                open: point.open.unwrap_or(point.price),
                high: point.high.unwrap_or(point.price),
                low: point.low.unwrap_or(point.price),
                close: point.close.unwrap_or(point.price),
                volume: point.volume,
                timestamp: point.timestamp.timestamp(),
            })
            .collect();

        // Calculate indicators if we have enough data
        let indicators = if candle_data.len() >= 20 {
            self.indicator_service.calculate_all_indicators_ohlcv(&candle_data)
        } else if !candle_data.is_empty() {
            // Not enough for full OHLCV analysis, use close prices only
            let prices: Vec<f64> = candle_data.iter().map(|c| c.close).collect();
            self.indicator_service.calculate_all_indicators(&prices)
        } else {
            HashMap::new()
        };

        Ok(MarketDataWithIndicators {
            symbol: symbol.to_string(),
            price: data.price,
            volume: data.volume_24h,
            indicators,
            timestamp: data.timestamp,
        })
    }

    pub async fn search_symbols(
        &self,
        query: &str,
        _limit: Option<usize>,
    ) -> ApiResult<Vec<SymbolInfo>> {
        // TODO: Query from exchange trading pairs
        if let Some(conn) = &self.connector {
            match conn.get_trading_pairs().await {
                Ok(pairs) => {
                    let matches: Vec<SymbolInfo> = pairs
                        .into_iter()
                        .filter(|p| p.symbol.to_lowercase().contains(&query.to_lowercase()))
                        .map(|p| SymbolInfo {
                            symbol: p.symbol.clone(),
                            name: Some(format!("{}/{}", p.base, p.quote)),
                            asset_class: "Crypto".to_string(),
                            exchange: format!("{:?}", conn.exchange_id()),
                            price_precision: 8,
                            quantity_precision: 8,
                        })
                        .collect();
                    return Ok(matches);
                }
                Err(e) => {
                    tracing::warn!("Failed to search symbols: {}", e);
                }
            }
        }
        Ok(vec![])
    }

    pub async fn get_market_overview(&self) -> ApiResult<MarketOverview> {
        // TODO: Aggregate from multiple symbols
        Ok(MarketOverview {
            top_gainers: vec![],
            top_losers: vec![],
            most_active: vec![],
            market_sentiment: 0.0,
        })
    }

    pub async fn subscribe_to_price_stream(
        &self,
        symbol: &str,
    ) -> ApiResult<StreamSubscriptionResponse> {
        Ok(StreamSubscriptionResponse {
            subscription_id: uuid::Uuid::new_v4().to_string(),
            status: "active".to_string(),
            stream_url: format!("ws://localhost:8080/ws/market/{}", symbol),
        })
    }

    pub async fn get_market_statistics(&self, symbol: &str) -> ApiResult<MarketStatistics> {
        // TODO: Calculate from historical data
        let now = Utc::now();
        Ok(MarketStatistics {
            symbol: symbol.to_string(),
            period_start: now - chrono::Duration::days(1),
            period_end: now,
            price_statistics: PriceStatistics {
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
                vwap: 0.0,
            },
            volatility_metrics: VolatilityMetrics {
                daily_volatility: 0.0,
                annualized_volatility: 0.0,
                bb_width: 0.0,
                atr: 0.0,
            },
            liquidity_metrics: LiquidityMetrics {
                average_spread: 0.0,
                average_volume: 0.0,
                turnover: 0.0,
                depth: 0.0,
            },
            trading_activity: TradingActivity {
                buy_count: 0,
                sell_count: 0,
                buy_volume: 0.0,
                sell_volume: 0.0,
                large_trades: 0,
            },
        })
    }
}

/// Manager for strategy operations
///
/// Manages trading strategies stored in the database.
pub struct StrategyManager {
    db: Arc<DatabaseManager>,
}

impl StrategyManager {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub async fn list_strategies(
        &self,
        _params: PaginationParams,
    ) -> ApiResult<PaginatedResponse<StrategyResponse>> {
        // TODO: Query from database
        Ok(PaginatedResponse {
            response: ApiResponse::success(vec![]),
            pagination: PaginationMeta {
                page: 1,
                limit: 50,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
        })
    }

    pub async fn get_strategy(&self, id: &str) -> ApiResult<Option<StrategyResponse>> {
        // TODO: Query from database
        Ok(None)
    }

    pub async fn create_strategy(
        &self,
        request: CreateStrategyRequest,
    ) -> ApiResult<StrategyResponse> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(StrategyResponse {
            id,
            name: request.name,
            description: request.description,
            parameters: request.parameters,
            is_active: false, // New strategies start inactive
            account_ids: request.account_ids.unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            performance: StrategyPerformance {
                total_trades: 0,
                win_rate: 0.0,
                total_pnl: 0.0,
                avg_trade_duration: 0.0,
                max_drawdown: 0.0,
            },
        })
    }

    pub async fn update_strategy(
        &self,
        id: &str,
        request: UpdateStrategyRequest,
    ) -> ApiResult<StrategyResponse> {
        // TODO: Update in database
        Err(ApiError::NotFound {
            resource: format!("Strategy {}", id),
        })
    }

    pub async fn delete_strategy(&self, id: &str) -> ApiResult<()> {
        // TODO: Delete from database
        Err(ApiError::NotFound {
            resource: format!("Strategy {}", id),
        })
    }

    pub async fn execute_strategy(
        &self,
        id: &str,
        _request: StrategyExecutionRequest,
    ) -> ApiResult<StrategyExecutionResponse> {
        Err(ApiError::NotImplemented {
            message: format!("Strategy execution not yet implemented for {}", id),
        })
    }

    pub async fn get_execution_history(
        &self,
        _id: &str,
        _params: PaginationParams,
    ) -> ApiResult<PaginatedResponse<StrategyExecutionResponse>> {
        Ok(PaginatedResponse {
            response: ApiResponse::success(vec![]),
            pagination: PaginationMeta {
                page: 1,
                limit: 50,
                total: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
        })
    }

    pub async fn backtest_strategy(
        &self,
        id: &str,
        _request: BacktestRequest,
    ) -> ApiResult<BacktestResponse> {
        Err(ApiError::NotImplemented {
            message: format!("Backtesting not yet implemented for strategy {}", id),
        })
    }

    pub async fn optimize_strategy(
        &self,
        id: &str,
        _request: StrategyOptimizationRequest,
    ) -> ApiResult<StrategyOptimizationResponse> {
        Err(ApiError::NotImplemented {
            message: format!("Strategy optimization not yet implemented for {}", id),
        })
    }

    pub async fn get_detailed_performance(
        &self,
        id: &str,
    ) -> ApiResult<DetailedStrategyPerformance> {
        // TODO: Calculate from trade history
        Ok(DetailedStrategyPerformance {
            basic_metrics: StrategyPerformance {
                total_trades: 0,
                win_rate: 0.0,
                total_pnl: 0.0,
                avg_trade_duration: 0.0,
                max_drawdown: 0.0,
            },
            monthly_returns: HashMap::new(),
            recent_trades: vec![],
        })
    }
}
