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

        Self { models }
    }

    /// Resolve pricing. Returns `None` if no matching model found.
    pub fn get_price(&self, model: &str) -> Option<ModelPricing> {
        let bare = model.split('[').next().unwrap_or(model);

        // Exact match
        if let Some(p) = self.models.get(bare) {
            return Some(p.clone());
        }

        // Fuzzy token match
        let input_tokens = model_tokens(bare);
        let mut best: Option<(&ModelPricing, usize)> = None;

        for (key, p) in &self.models {
            let key_tokens = model_tokens(key);
            let overlap = input_tokens
                .iter()
                .filter(|t| key_tokens.contains(t))
                .count();
            if overlap >= 2 && best.is_none_or(|(_, prev_score)| overlap > prev_score) {
                best = Some((p, overlap));
            }
        }

        best.map(|(p, _)| p.clone())
    }

    /// Estimate cost for a session snapshot. Returns 0.0 if model not found.
    pub fn estimate_cost(&self, snapshot: &SessionSnapshot) -> f64 {
        let Some(pricing) = self.get_price(&snapshot.model) else {
            return 0.0;
        };

        // Clamp cached to input_tokens to handle cross-source merge inconsistency
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
    let engine = PricingEngine::new(&crate::config::PricingConfig::default());
    let Some(pricing) = engine.get_price(model) else {
        return 0.0;
    };

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

/// Split a model name into lowercase tokens for fuzzy matching.
/// e.g. "claude-4.6-opus" → {"claude", "4", "6", "opus"}
/// e.g. "claude-opus-4-6[1m]" → {"claude", "opus", "4", "6"}
fn model_tokens(name: &str) -> Vec<String> {
    // Strip [...] suffix first
    let bare = name.split('[').next().unwrap_or(name);
    bare.to_lowercase()
        .split(['-', '_', '.'])
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}
