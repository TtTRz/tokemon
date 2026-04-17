use std::collections::HashMap;

use crate::config::PricingConfig;
use crate::model::SessionSnapshot;

/// Pricing for a single model.
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_write_per_mtok: Option<f64>,
    pub cache_read_per_mtok: Option<f64>,
}

/// Engine that resolves model prices and estimates cost at render time.
pub struct PricingEngine {
    /// Model pricing from config (includes defaults + user overrides)
    models: HashMap<String, ModelPricing>,
    /// Fallback for unknown models
    default_input: f64,
    default_output: f64,
}

impl PricingEngine {
    pub fn new(config: &PricingConfig) -> Self {
        let mut models = HashMap::new();
        for (model, p) in &config.models {
            models.insert(
                model.clone(),
                ModelPricing {
                    input_per_mtok: p.input,
                    output_per_mtok: p.output,
                    cache_write_per_mtok: p.cache_write,
                    cache_read_per_mtok: p.cache_read,
                },
            );
        }

        Self {
            models,
            default_input: config.default_input,
            default_output: config.default_output,
        }
    }

    /// Resolve pricing: exact match > prefix match > fallback.
    pub fn get_price(&self, model: &str) -> ModelPricing {
        if let Some(p) = self.models.get(model) {
            return p.clone();
        }
        // Try prefix match (e.g. "claude-sonnet-4" matches "claude-sonnet-4-20250514")
        for (key, p) in &self.models {
            if model.starts_with(key) || key.starts_with(model) {
                return p.clone();
            }
        }
        ModelPricing {
            input_per_mtok: self.default_input,
            output_per_mtok: self.default_output,
            cache_write_per_mtok: None,
            cache_read_per_mtok: None,
        }
    }

    /// Estimate cost for a session snapshot (called at render time).
    ///
    /// `input_tokens` is the total input count (including cached portions).
    /// We subtract cache tokens to avoid double-counting, then price each
    /// bucket at its own rate. If no cache-specific pricing is configured,
    /// cached tokens fall back to the regular input rate.
    pub fn estimate_cost(&self, snapshot: &SessionSnapshot) -> f64 {
        let pricing = self.get_price(&snapshot.model);

        // Clamp total cached to input_tokens to avoid negative non_cached_input
        // (can happen when merge() updates fields independently from different sources)
        let total_cached = (snapshot.cache_creation_tokens + snapshot.cache_read_tokens)
            .min(snapshot.input_tokens);
        let non_cached_input = snapshot.input_tokens - total_cached;

        let input_cost = non_cached_input as f64 * pricing.input_per_mtok / 1_000_000.0;
        let output_cost = snapshot.output_tokens as f64 * pricing.output_per_mtok / 1_000_000.0;
        let cache_write_cost = snapshot.cache_creation_tokens as f64
            * pricing
                .cache_write_per_mtok
                .unwrap_or(pricing.input_per_mtok)
            / 1_000_000.0;
        let cache_read_cost = snapshot.cache_read_tokens as f64
            * pricing.cache_read_per_mtok.unwrap_or(0.0)
            / 1_000_000.0;
        input_cost + output_cost + cache_write_cost + cache_read_cost
    }
}

/// Estimate cost for a single API turn using builtin pricing (no PricingEngine instance needed).
/// Used by providers during JSONL parsing when PricingEngine is not available.
/// `input_tokens` includes cache tokens — we subtract them to get non-cached input.
pub fn estimate_turn_cost_builtin(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation: u64,
    cache_read: u64,
) -> f64 {
    // Resolve pricing from builtins (same data as PricingEngine::builtin_prices)
    let engine = PricingEngine::new(&crate::config::PricingConfig::default());
    let pricing = engine.get_price(model);

    let total_cached = (cache_creation + cache_read).min(input_tokens);
    let non_cached = input_tokens - total_cached;

    let input_cost = non_cached as f64 * pricing.input_per_mtok / 1_000_000.0;
    let output_cost = output_tokens as f64 * pricing.output_per_mtok / 1_000_000.0;
    let write_cost = cache_creation as f64
        * pricing
            .cache_write_per_mtok
            .unwrap_or(pricing.input_per_mtok)
        / 1_000_000.0;
    let read_cost = cache_read as f64 * pricing.cache_read_per_mtok.unwrap_or(0.0) / 1_000_000.0;

    input_cost + output_cost + write_cost + read_cost
}
