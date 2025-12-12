use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use tracing::info;

pub struct SafetyValidator {
    max_position_size: Decimal,
    daily_loss_limit: Decimal,
    dry_run: bool,
}

impl SafetyValidator {
    pub fn new(max_position_size: Decimal, daily_loss_limit: Decimal, dry_run: bool) -> Self {
        Self {
            max_position_size,
            daily_loss_limit,
            dry_run,
        }
    }

    pub fn check_trade(
        &self,
        symbol: &str,
        quantity: Decimal,
        estimated_value: Decimal,
    ) -> Result<()> {
        if self.dry_run {
            info!(
                "Dry run mode active: skipping strict safety checks for {}",
                symbol
            );
            return Ok(());
        }

        if estimated_value > self.max_position_size {
            return Err(anyhow!(
                "Order value {} exceeds max position size {}",
                estimated_value,
                self.max_position_size
            ));
        }

        // Placeholder for daily loss check (requires state)
        // if current_daily_loss + potential_loss > self.daily_loss_limit { ... }

        Ok(())
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}
