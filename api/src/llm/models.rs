//! Production-grade LLM model registry for ninja-gekko
//! All model IDs verified against OpenRouter catalog as of December 2025

use serde::{Deserialize, Serialize};

/// LLM model definition for OpenRouter API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    /// OpenRouter model identifier (e.g., "nvidia/nemotron-nano-12b-v2-vl:free")
    pub id: &'static str,
    /// Human-readable display name for UI
    pub display_name: &'static str,
    /// Model provider (e.g., "NVIDIA", "DeepSeek", "Google")
    pub provider: &'static str,
    /// Maximum context window in tokens
    pub context_window: u32,
    /// Brief description of model specialization
    pub specialization: &'static str,
}

/// Complete model registry with 12 production-ready options
/// Optimized for: financial reasoning, mathematical analysis, time-series forecasting
pub const MODEL_REGISTRY: &[LlmModel] = &[
    // ═══════════════════════════════════════════════════════════════════
    // DEFAULT: Free tier, vision-capable, solid general reasoning
    // ═══════════════════════════════════════════════════════════════════
    LlmModel {
        id: "nvidia/nemotron-nano-12b-v2-vl:free",
        display_name: "Nemotron Nano 12B (Free)",
        provider: "NVIDIA",
        context_window: 8192,
        specialization: "General reasoning with vision, free tier default",
    },

    // ═══════════════════════════════════════════════════════════════════
    // PREMIUM REASONING: Best-in-class for complex financial analysis
    // ═══════════════════════════════════════════════════════════════════
    LlmModel {
        id: "deepseek/deepseek-v3.2-20251201",
        display_name: "DeepSeek V3.2",
        provider: "DeepSeek",
        context_window: 128_000,
        specialization: "GPT-5 class reasoning, tool use, agentic workflows",
    },
    LlmModel {
        id: "deepseek/deepseek-r1-0528",
        display_name: "DeepSeek R1 (May 2028)",
        provider: "DeepSeek",
        context_window: 128_000,
        specialization: "SOTA math/logic, IMO gold medal, multi-step reasoning",
    },
    LlmModel {
        id: "google/gemini-3-pro-preview-20251117",
        display_name: "Gemini 3 Pro",
        provider: "Google",
        context_window: 1_000_000,
        specialization: "1M context, multimodal, advanced reasoning",
    },
    LlmModel {
        id: "anthropic/claude-4.5-sonnet-20250929",
        display_name: "Claude 4.5 Sonnet",
        provider: "Anthropic",
        context_window: 200_000,
        specialization: "Complex planning, coding, safety-aligned reasoning",
    },

    // ═══════════════════════════════════════════════════════════════════
    // MATHEMATICAL SPECIALISTS: Quantitative finance, statistics, proofs
    // ═══════════════════════════════════════════════════════════════════
    LlmModel {
        id: "microsoft/phi-4-reasoning",
        display_name: "Phi-4 Reasoning",
        provider: "Microsoft",
        context_window: 32_000,
        specialization: "14B math/science/code, AIME/OmniMath SOTA, MIT licensed",
    },
    LlmModel {
        id: "deepseek/deepseek-r1-distill-qwen-32b",
        display_name: "R1 Distill Qwen 32B",
        provider: "DeepSeek",
        context_window: 128_000,
        specialization: "94.3% MATH-500, beats o1-mini, strong quantitative",
    },
    LlmModel {
        id: "deepseek/deepseek-prover-v2",
        display_name: "DeepSeek Prover V2",
        provider: "DeepSeek",
        context_window: 163_000,
        specialization: "Formal theorem proving (Lean 4), mathematical proofs",
    },
    LlmModel {
        id: "qwen/qwen3-max",
        display_name: "Qwen3 Max",
        provider: "Alibaba",
        context_window: 256_000,
        specialization: "Math, logic, science, RAG-optimized, 100+ languages",
    },

    // ═══════════════════════════════════════════════════════════════════
    // COST-EFFICIENT: High performance at lower token costs
    // ═══════════════════════════════════════════════════════════════════
    LlmModel {
        id: "google/gemini-2.5-flash",
        display_name: "Gemini 2.5 Flash",
        provider: "Google",
        context_window: 1_000_000,
        specialization: "Fast inference, cost-effective, strong reasoning",
    },
    LlmModel {
        id: "deepseek/deepseek-r1-0528-qwen3-8b:free",
        display_name: "R1 Distill Qwen3 8B (Free)",
        provider: "DeepSeek",
        context_window: 128_000,
        specialization: "Free tier, ties 235B on AIME 2024, compact reasoning",
    },

    // ═══════════════════════════════════════════════════════════════════
    // CODE GENERATION: For strategy implementation, system automation
    // ═══════════════════════════════════════════════════════════════════
    LlmModel {
        id: "qwen/qwen3-coder-480b-a35b-07-25",
        display_name: "Qwen3 Coder 480B",
        provider: "Alibaba",
        context_window: 131_000,
        specialization: "Autonomous coding agent, tool calling, MoE architecture",
    },
];

/// Returns default model for Gordon chat interface
pub fn default_model() -> &'static LlmModel {
    &MODEL_REGISTRY[0] // nvidia/nemotron-nano-12b-v2-vl:free
}

/// Returns model by OpenRouter ID
pub fn get_model(id: &str) -> Option<&'static LlmModel> {
    MODEL_REGISTRY.iter().find(|m| m.id == id)
}

/// Returns models optimized for mathematical/quantitative tasks
pub fn quantitative_models() -> impl Iterator<Item = &'static LlmModel> {
    MODEL_REGISTRY.iter().filter(|m| {
        m.specialization.contains("math") ||
        m.specialization.contains("quantitative") ||
        m.specialization.contains("reasoning")
    })
}

/// Returns free-tier models only
pub fn free_models() -> impl Iterator<Item = &'static LlmModel> {
    MODEL_REGISTRY.iter().filter(|m| m.id.ends_with(":free"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_is_free() {
        let model = default_model();
        assert!(model.id.ends_with(":free"), "Default model should be free tier");
    }

    #[test]
    fn test_all_models_have_valid_context() {
        for model in MODEL_REGISTRY {
            assert!(model.context_window >= 8192, "Context window too small: {}", model.id);
        }
    }

    #[test]
    fn test_get_model_found() {
        let model = get_model("deepseek/deepseek-v3.2-20251201");
        assert!(model.is_some());
        assert_eq!(model.unwrap().provider, "DeepSeek");
    }

    #[test]
    fn test_get_model_not_found() {
        let model = get_model("nonexistent/model");
        assert!(model.is_none());
    }

    #[test]
    fn test_quantitative_models_not_empty() {
        let count = quantitative_models().count();
        assert!(count >= 5, "Should have at least 5 quantitative models");
    }
}
