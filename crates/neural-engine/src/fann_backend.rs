//! FANN Backend for Neural Engine
//!
//! This module provides integration with the ruv-FANN library for neural network
//! inference in the arbitrage trading system. It wraps FANN networks and provides
//! a unified interface for model loading and forward propagation.

use crate::{NeuralError, NeuralResult};
use ruv_fann::{ActivationFunction, Network, NetworkBuilder};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

/// Model metadata for FANN networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FannModelMetadata {
    /// Model version identifier
    pub version: String,
    /// Number of input neurons
    pub input_size: usize,
    /// Number of output neurons
    pub output_size: usize,
    /// Hidden layer sizes
    pub hidden_layers: Vec<usize>,
    /// Model purpose/description
    pub description: String,
    /// Training date (ISO 8601)
    pub training_date: Option<String>,
    /// Model accuracy from validation set
    pub accuracy: Option<f64>,
}

impl Default for FannModelMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            input_size: 0,
            output_size: 0,
            hidden_layers: vec![],
            description: String::new(),
            training_date: None,
            accuracy: None,
        }
    }
}

/// FANN Model wrapper for neural inference
///
/// Wraps a ruv-FANN Network and provides methods for loading models
/// from files and running inference.
pub struct FannModel {
    /// The underlying FANN network
    network: Network<f64>,
    /// Model metadata
    pub metadata: FannModelMetadata,
    /// Model file path (if loaded from file)
    pub file_path: Option<String>,
}

impl FannModel {
    /// Create a new FANN model with specified architecture
    ///
    /// # Arguments
    /// * `input_size` - Number of input neurons
    /// * `hidden_sizes` - Vector of hidden layer sizes
    /// * `output_size` - Number of output neurons
    ///
    /// # Returns
    /// A new FannModel with random initialization
    pub fn new(input_size: usize, hidden_sizes: &[usize], output_size: usize) -> NeuralResult<Self> {
        info!(
            "🧠 Creating FANN model: {} inputs, {:?} hidden, {} outputs",
            input_size, hidden_sizes, output_size
        );

        let mut builder = NetworkBuilder::new().input_layer(input_size);

        for &size in hidden_sizes {
            builder = builder.hidden_layer_with_activation(size, ActivationFunction::Sigmoid, 0.5);
        }

        let network = builder
            .output_layer(output_size)
            .connection_rate(1.0)
            .build();

        let metadata = FannModelMetadata {
            version: "1.0.0".to_string(),
            input_size,
            output_size,
            hidden_layers: hidden_sizes.to_vec(),
            description: "Newly created FANN model".to_string(),
            training_date: None,
            accuracy: None,
        };

        Ok(Self {
            network,
            metadata,
            file_path: None,
        })
    }

    /// Load a FANN model from a .net file
    ///
    /// This attempts to load model weights and architecture from a FANN-compatible
    /// file format. If the file doesn't exist, returns an error.
    ///
    /// # Arguments
    /// * `path` - Path to the .net model file
    /// * `expected_input_size` - Expected number of input neurons (for validation)
    /// * `expected_output_size` - Expected number of output neurons (for validation)
    #[allow(dead_code)]
    pub fn load_from_file(
        path: &str,
        expected_input_size: usize,
        expected_output_size: usize,
    ) -> NeuralResult<Self> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(NeuralError::ModelNotFound(format!(
                "Model file not found: {}",
                path
            )));
        }

        info!("📂 Loading FANN model from: {}", path);

        // For now, create a model with the expected dimensions
        // In production, this would parse the .net file format
        warn!("FANN file loading not fully implemented, creating placeholder network");

        let model = Self::new(expected_input_size, &[16, 8], expected_output_size)?;

        Ok(Self {
            file_path: Some(path.to_string()),
            ..model
        })
    }

    /// Create a volatility prediction model
    ///
    /// Creates a FANN network configured for predicting price volatility.
    /// Input features: [price, high, low, volume, avg_volume, bid, ask]
    /// Output: [volatility_1m, volatility_5m, volatility_15m, confidence]
    pub fn create_volatility_model() -> NeuralResult<Self> {
        let input_size = 7; // price, high, low, volume, avg_volume, bid, ask
        let hidden_sizes = [32, 16];
        let output_size = 4; // volatility_1m, volatility_5m, volatility_15m, confidence

        let mut model = Self::new(input_size, &hidden_sizes, output_size)?;
        model.metadata.description = "Volatility prediction model for arbitrage trading".to_string();

        Ok(model)
    }

    /// Create an arbitrage detection model
    ///
    /// Creates a FANN network configured for detecting cross-exchange arbitrage opportunities.
    /// Input features: [primary_price, secondary_price, spread, primary_volume, secondary_volume,
    ///                  primary_bid, primary_ask, secondary_bid, secondary_ask]
    /// Output: [spread_1m, spread_5m, arb_probability, expected_profit, confidence]
    pub fn create_arbitrage_model() -> NeuralResult<Self> {
        let input_size = 9;
        let hidden_sizes = [48, 24, 12];
        let output_size = 5;

        let mut model = Self::new(input_size, &hidden_sizes, output_size)?;
        model.metadata.description = "Cross-exchange arbitrage detection model".to_string();

        Ok(model)
    }

    /// Create a risk assessment model
    ///
    /// Creates a FANN network configured for assessing arbitrage risk.
    /// Input features: [position_size, volatility, spread, volume_ratio, time_factor]
    /// Output: [overall_risk, liquidity_risk, execution_risk, max_position, confidence]
    pub fn create_risk_model() -> NeuralResult<Self> {
        let input_size = 5;
        let hidden_sizes = [24, 12];
        let output_size = 5;

        let mut model = Self::new(input_size, &hidden_sizes, output_size)?;
        model.metadata.description = "Risk assessment model for arbitrage trading".to_string();

        Ok(model)
    }

    /// Run forward propagation on the network
    ///
    /// # Arguments
    /// * `input` - Input feature vector (must match input_size)
    ///
    /// # Returns
    /// Output vector from the network
    pub fn run(&mut self, input: &[f64]) -> NeuralResult<Vec<f64>> {
        if input.len() != self.metadata.input_size {
            return Err(NeuralError::InvalidInput(format!(
                "Expected {} inputs, got {}",
                self.metadata.input_size,
                input.len()
            )));
        }

        debug!(
            "🔮 Running FANN inference with {} inputs",
            input.len()
        );

        let output = self.network.run(input);

        if output.is_empty() {
            return Err(NeuralError::InferenceFailed(
                "Network returned empty output".to_string(),
            ));
        }

        debug!("📊 FANN output: {:?}", output);

        Ok(output)
    }

    /// Validate model dimensions
    ///
    /// Checks that the model's input/output sizes match expected dimensions.
    pub fn validate_dimensions(
        &self,
        expected_input: usize,
        expected_output: usize,
    ) -> NeuralResult<()> {
        if self.metadata.input_size != expected_input {
            return Err(NeuralError::InvalidInput(format!(
                "Model input size mismatch: expected {}, got {}",
                expected_input, self.metadata.input_size
            )));
        }

        if self.metadata.output_size != expected_output {
            return Err(NeuralError::InvalidInput(format!(
                "Model output size mismatch: expected {}, got {}",
                expected_output, self.metadata.output_size
            )));
        }

        Ok(())
    }

    /// Get the number of input neurons
    pub fn input_size(&self) -> usize {
        self.metadata.input_size
    }

    /// Get the number of output neurons
    pub fn output_size(&self) -> usize {
        self.metadata.output_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_volatility_model() {
        let model = FannModel::create_volatility_model();
        assert!(model.is_ok());

        let model = model.unwrap();
        assert_eq!(model.input_size(), 7);
        assert_eq!(model.output_size(), 4);
    }

    #[test]
    fn test_create_arbitrage_model() {
        let model = FannModel::create_arbitrage_model();
        assert!(model.is_ok());

        let model = model.unwrap();
        assert_eq!(model.input_size(), 9);
        assert_eq!(model.output_size(), 5);
    }

    #[test]
    fn test_create_risk_model() {
        let model = FannModel::create_risk_model();
        assert!(model.is_ok());

        let model = model.unwrap();
        assert_eq!(model.input_size(), 5);
        assert_eq!(model.output_size(), 5);
    }

    #[test]
    fn test_run_inference() {
        let mut model = FannModel::create_volatility_model().unwrap();
        let input = vec![100.0, 101.0, 99.0, 1000.0, 800.0, 99.9, 100.1];

        let result = model.run(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_invalid_input_size() {
        let mut model = FannModel::create_volatility_model().unwrap();
        let input = vec![100.0, 101.0, 99.0]; // Too few inputs

        let result = model.run(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dimensions() {
        let model = FannModel::create_volatility_model().unwrap();

        // Valid dimensions
        assert!(model.validate_dimensions(7, 4).is_ok());

        // Invalid input size
        assert!(model.validate_dimensions(8, 4).is_err());

        // Invalid output size
        assert!(model.validate_dimensions(7, 3).is_err());
    }
}
