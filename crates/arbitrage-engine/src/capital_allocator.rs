//! Capital Allocator - Aggressive Cross-Exchange Fund Management
//!
//! This module implements dynamic capital allocation and reallocation across
//! multiple exchanges to maximize arbitrage opportunities and returns.

use crate::{AllocationPriority, AllocationRequest, ArbitrageError, ArbitrageResult};
use exchange_connectors::{
    Balance, ExchangeConnector, ExchangeId, TransferRequest, TransferUrgency,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Capital allocation strategy
#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    /// Equal distribution across all exchanges
    Balanced,
    /// Concentrate capital on highest opportunity exchanges
    Aggressive,
    /// Custom weights per exchange
    Weighted(HashMap<ExchangeId, f64>),
}

/// Capital allocator for cross-exchange fund management
pub struct CapitalAllocator {
    exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>,
    allocation_strategy: Arc<RwLock<AllocationStrategy>>,
    pending_allocations: Arc<RwLock<HashMap<Uuid, AllocationRequest>>>,
    target_allocations: Arc<RwLock<HashMap<ExchangeId, Decimal>>>,
    current_balances: Arc<RwLock<HashMap<ExchangeId, Vec<Balance>>>>,
}

impl CapitalAllocator {
    /// Create a new capital allocator
    pub fn new(exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>) -> Self {
        Self {
            exchanges,
            allocation_strategy: Arc::new(RwLock::new(AllocationStrategy::Aggressive)),
            pending_allocations: Arc::new(RwLock::new(HashMap::new())),
            target_allocations: Arc::new(RwLock::new(HashMap::new())),
            current_balances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize capital allocator by fetching current balances
    pub async fn initialize(&self) -> ArbitrageResult<()> {
        info!(
            "💰 Initializing capital allocator across {} exchanges",
            self.exchanges.len()
        );

        let mut balances = self.current_balances.write().await;

        for (exchange_id, connector) in &self.exchanges {
            match connector.get_balances().await {
                Ok(exchange_balances) => {
                    info!(
                        "💵 Loaded {} currency balances from {:?}",
                        exchange_balances.len(),
                        exchange_id
                    );
                    balances.insert(*exchange_id, exchange_balances);
                }
                Err(e) => {
                    warn!("Failed to fetch balances from {:?}: {}", exchange_id, e);
                    return Err(ArbitrageError::Exchange(format!(
                        "Failed to initialize balances for {:?}: {}",
                        exchange_id, e
                    )));
                }
            }
        }

        // Calculate initial target allocations
        self.calculate_target_allocations().await?;

        info!("✅ Capital allocator initialized successfully");
        Ok(())
    }

    /// Request capital allocation for arbitrage opportunity
    pub async fn request_allocation(
        &self,
        from_exchange: ExchangeId,
        to_exchange: ExchangeId,
        currency: String,
        amount: Decimal,
        priority: AllocationPriority,
        reason: String,
    ) -> ArbitrageResult<Uuid> {
        let request = AllocationRequest {
            id: Uuid::new_v4(),
            from_exchange,
            to_exchange,
            currency: currency.clone(),
            amount,
            priority,
            reason,
            requested_at: chrono::Utc::now(),
            deadline: chrono::Utc::now()
                + chrono::Duration::minutes(match priority {
                    AllocationPriority::Emergency => 1,
                    AllocationPriority::Critical => 5,
                    AllocationPriority::High => 15,
                    AllocationPriority::Normal => 60,
                    AllocationPriority::Low => 240,
                }),
        };

        info!(
            "🎯 Capital allocation requested: {} {} from {:?} to {:?} (Priority: {:?})",
            amount, currency, from_exchange, to_exchange, priority
        );

        let mut pending = self.pending_allocations.write().await;
        let request_id = request.id;
        pending.insert(request_id, request);

        Ok(request_id)
    }

    /// Process pending allocations and execute transfers
    pub async fn rebalance_capital(&self) -> ArbitrageResult<()> {
        debug!("⚖️ Starting capital rebalancing cycle");

        // Update current balances
        self.update_current_balances().await?;

        // Process high-priority allocations first
        self.process_pending_allocations().await?;

        // Perform strategic rebalancing
        self.perform_strategic_rebalancing().await?;

        debug!("✅ Capital rebalancing cycle completed");
        Ok(())
    }

    /// Set allocation strategy
    pub async fn set_allocation_strategy(&self, strategy: AllocationStrategy) {
        let mut current_strategy = self.allocation_strategy.write().await;
        *current_strategy = strategy;

        // Recalculate target allocations with new strategy
        drop(current_strategy);
        if let Err(e) = self.calculate_target_allocations().await {
            warn!("Failed to recalculate target allocations: {}", e);
        }
    }

    /// Get current capital distribution across exchanges
    pub async fn get_capital_distribution(&self) -> HashMap<ExchangeId, HashMap<String, Decimal>> {
        let balances = self.current_balances.read().await;
        let mut distribution = HashMap::new();

        for (exchange_id, exchange_balances) in balances.iter() {
            let mut currency_balances = HashMap::new();
            for balance in exchange_balances {
                currency_balances.insert(balance.currency.clone(), balance.available);
            }
            distribution.insert(*exchange_id, currency_balances);
        }

        distribution
    }

    /// Emergency capital reallocation (Gekko mode activation)
    pub async fn emergency_reallocation(
        &self,
        target_exchange: ExchangeId,
        currency: String,
        percentage: f64,
    ) -> ArbitrageResult<Vec<Uuid>> {
        error!("🚨 EMERGENCY CAPITAL REALLOCATION TRIGGERED 🚨");
        error!(
            "Target: {:?}, Currency: {}, Percentage: {}%",
            target_exchange,
            currency,
            percentage * 100.0
        );

        let mut allocation_requests = Vec::new();
        let balances = self.current_balances.read().await;

        for (exchange_id, exchange_balances) in balances.iter() {
            if *exchange_id == target_exchange {
                continue; // Don't transfer to itself
            }

            // Find the currency balance
            if let Some(balance) = exchange_balances.iter().find(|b| b.currency == currency) {
                let transfer_amount =
                    balance.available * Decimal::from_f64_retain(percentage).unwrap_or_default();

                if transfer_amount > Decimal::ZERO {
                    let request_id = self
                        .request_allocation(
                            *exchange_id,
                            target_exchange,
                            currency.clone(),
                            transfer_amount,
                            AllocationPriority::Emergency,
                            "Emergency Gekko-style capital reallocation".to_string(),
                        )
                        .await?;

                    allocation_requests.push(request_id);

                    error!(
                        "💀 Emergency transfer: {} {} from {:?} to {:?}",
                        transfer_amount, currency, exchange_id, target_exchange
                    );
                }
            }
        }

        error!(
            "🎯 Emergency reallocation complete: {} transfers initiated",
            allocation_requests.len()
        );
        Ok(allocation_requests)
    }

    // Private implementation methods

    /// Update current balances from all exchanges
    async fn update_current_balances(&self) -> ArbitrageResult<()> {
        let mut balances = self.current_balances.write().await;

        for (exchange_id, connector) in &self.exchanges {
            match connector.get_balances().await {
                Ok(exchange_balances) => {
                    balances.insert(*exchange_id, exchange_balances);
                }
                Err(e) => {
                    warn!("Failed to update balances for {:?}: {}", exchange_id, e);
                }
            }
        }

        Ok(())
    }

    /// Process pending allocation requests by priority
    async fn process_pending_allocations(&self) -> ArbitrageResult<()> {
        let mut pending = self.pending_allocations.write().await;
        let mut completed_requests = Vec::new();

        // Sort by priority and deadline
        let mut sorted_requests: Vec<AllocationRequest> = pending.values().cloned().collect();
        sorted_requests.sort_by(|a, b| {
            let priority_order = |p: &AllocationPriority| match p {
                AllocationPriority::Emergency => 0,
                AllocationPriority::Critical => 1,
                AllocationPriority::High => 2,
                AllocationPriority::Normal => 3,
                AllocationPriority::Low => 4,
            };

            priority_order(&a.priority)
                .cmp(&priority_order(&b.priority))
                .then_with(|| a.deadline.cmp(&b.deadline))
        });

        for request in sorted_requests {
            // Check if request has expired
            if chrono::Utc::now() > request.deadline {
                warn!("⏰ Allocation request {} expired", request.id);
                completed_requests.push(request.id);
                continue;
            }

            // Execute the allocation
            match self.execute_allocation(&request).await {
                Ok(_) => {
                    info!(
                        "✅ Allocation executed: {} {} from {:?} to {:?}",
                        request.amount,
                        request.currency,
                        request.from_exchange,
                        request.to_exchange
                    );
                    completed_requests.push(request.id);
                }
                Err(e) => {
                    warn!("❌ Allocation failed for {}: {}", request.id, e);
                    // Keep in pending for retry unless it's a permanent failure
                    if matches!(e, ArbitrageError::InsufficientCapital { .. }) {
                        completed_requests.push(request.id);
                    }
                }
            }
        }

        // Remove completed requests
        for request_id in completed_requests {
            pending.remove(&request_id);
        }

        Ok(())
    }

    /// Execute a specific allocation request
    async fn execute_allocation(&self, request: &AllocationRequest) -> ArbitrageResult<()> {
        // Check if we have sufficient balance
        let balances = self.current_balances.read().await;

        if let Some(from_balances) = balances.get(&request.from_exchange) {
            if let Some(balance) = from_balances
                .iter()
                .find(|b| b.currency == request.currency)
            {
                if balance.available < request.amount {
                    return Err(ArbitrageError::InsufficientCapital {
                        required: request.amount,
                        available: balance.available,
                    });
                }
            } else {
                return Err(ArbitrageError::InsufficientCapital {
                    required: request.amount,
                    available: Decimal::ZERO,
                });
            }
        }

        // Create transfer request
        if let Some(from_connector) = self.exchanges.get(&request.from_exchange) {
            let transfer_urgency = match request.priority {
                AllocationPriority::Emergency => TransferUrgency::Critical,
                AllocationPriority::Critical => TransferUrgency::High,
                AllocationPriority::High => TransferUrgency::Normal,
                AllocationPriority::Normal => TransferUrgency::Normal,
                AllocationPriority::Low => TransferUrgency::Low,
            };

            let transfer_request = TransferRequest {
                id: Uuid::new_v4(),
                from_exchange: request.from_exchange,
                to_exchange: request.to_exchange,
                currency: request.currency.clone(),
                amount: request.amount,
                urgency: transfer_urgency,
            };

            match from_connector.transfer_funds(transfer_request).await {
                Ok(transfer_id) => {
                    info!(
                        "💸 Transfer initiated: {} ({})",
                        transfer_id, request.reason
                    );
                    Ok(())
                }
                Err(e) => Err(ArbitrageError::Exchange(format!("Transfer failed: {}", e))),
            }
        } else {
            Err(ArbitrageError::Configuration(format!(
                "Exchange connector not found: {:?}",
                request.from_exchange
            )))
        }
    }

    /// Perform strategic rebalancing based on current strategy
    async fn perform_strategic_rebalancing(&self) -> ArbitrageResult<()> {
        let strategy = self.allocation_strategy.read().await;
        let targets = self.target_allocations.read().await;
        let balances = self.current_balances.read().await;

        // Implement strategic rebalancing logic based on current strategy
        match &*strategy {
            AllocationStrategy::Aggressive => {
                // Concentrate capital on highest opportunity exchanges
                // This would involve more complex logic based on current opportunities
                debug!("🔥 Aggressive rebalancing strategy active");
            }
            AllocationStrategy::Balanced => {
                // Ensure roughly equal distribution
                debug!("⚖️ Balanced rebalancing strategy active");
            }
            AllocationStrategy::Weighted(weights) => {
                // Use custom weights
                debug!(
                    "🎯 Weighted rebalancing strategy active with {} weights",
                    weights.len()
                );
            }
        }

        Ok(())
    }

    /// Calculate target allocations based on current strategy
    async fn calculate_target_allocations(&self) -> ArbitrageResult<()> {
        let strategy = self.allocation_strategy.read().await;
        let mut targets = self.target_allocations.write().await;

        let total_exchanges = self.exchanges.len() as f64;

        match &*strategy {
            AllocationStrategy::Balanced => {
                let per_exchange =
                    Decimal::from_f64_retain(1.0 / total_exchanges).unwrap_or_default();
                for exchange_id in self.exchanges.keys() {
                    targets.insert(*exchange_id, per_exchange);
                }
            }
            AllocationStrategy::Aggressive => {
                // Set higher allocations for exchanges with more opportunities
                // This is simplified - real implementation would consider current opportunities
                for exchange_id in self.exchanges.keys() {
                    targets.insert(
                        *exchange_id,
                        Decimal::from_f64_retain(0.4).unwrap_or_default(),
                    );
                }
            }
            AllocationStrategy::Weighted(weights) => {
                for (exchange_id, weight) in weights {
                    targets.insert(
                        *exchange_id,
                        Decimal::from_f64_retain(*weight).unwrap_or_default(),
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_request_creation() {
        let request = AllocationRequest {
            id: Uuid::new_v4(),
            from_exchange: ExchangeId::Kraken,
            to_exchange: ExchangeId::BinanceUs,
            currency: "USD".to_string(),
            amount: Decimal::new(10000, 0),
            priority: AllocationPriority::High,
            reason: "High-profit arbitrage opportunity".to_string(),
            requested_at: chrono::Utc::now(),
            deadline: chrono::Utc::now() + chrono::Duration::minutes(15),
        };

        assert_eq!(request.currency, "USD");
        assert_eq!(request.priority, AllocationPriority::High);
        assert_eq!(request.amount, Decimal::new(10000, 0));
    }

    #[test]
    fn test_allocation_strategy_variants() {
        let _balanced = AllocationStrategy::Balanced;
        let _aggressive = AllocationStrategy::Aggressive;

        let mut weights = HashMap::new();
        weights.insert(ExchangeId::Kraken, 0.4);
        weights.insert(ExchangeId::BinanceUs, 0.6);
        let _weighted = AllocationStrategy::Weighted(weights);
    }
}
