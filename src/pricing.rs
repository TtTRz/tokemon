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
    /// User overrides from config
    user_overrides: HashMap<String, ModelPricing>,
    /// Built-in defaults for well-known models
    builtins: HashMap<String, ModelPricing>,
    /// Fallback for unknown models
    default_input: f64,
    default_output: f64,
}

impl PricingEngine {
    pub fn new(config: &PricingConfig) -> Self {
        let mut user_overrides = HashMap::new();
        for (model, p) in &config.models {
            user_overrides.insert(
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
            user_overrides,
            builtins: Self::builtin_prices(),
            default_input: config.default_input,
            default_output: config.default_output,
        }
    }

    /// Resolve pricing: user override > builtin > fallback.
    pub fn get_price(&self, model: &str) -> ModelPricing {
        if let Some(p) = self.user_overrides.get(model) {
            return p.clone();
        }
        // Try prefix match for builtins (e.g. "claude-sonnet-4" matches "claude-sonnet-4-*")
        for (key, p) in &self.builtins {
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

        let total_cached = snapshot.cache_creation_tokens + snapshot.cache_read_tokens;
        let non_cached_input = snapshot.input_tokens.saturating_sub(total_cached);

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

    fn builtin_prices() -> HashMap<String, ModelPricing> {
        let mut m = HashMap::new();
        m.insert(
            "claude-sonnet-4".into(),
            ModelPricing {
                input_per_mtok: 3.0,
                output_per_mtok: 15.0,
                cache_write_per_mtok: Some(3.75),
                cache_read_per_mtok: Some(0.30),
            },
        );
        m.insert(
            "claude-4.6-opus".into(),
            ModelPricing {
                input_per_mtok: 5.0,
                output_per_mtok: 25.0,
                cache_write_per_mtok: Some(6.25),
                cache_read_per_mtok: Some(0.50),
            },
        );
        m.insert(
            "claude-haiku-3.5".into(),
            ModelPricing {
                input_per_mtok: 0.80,
                output_per_mtok: 4.0,
                cache_write_per_mtok: Some(1.0),
                cache_read_per_mtok: Some(0.08),
            },
        );
        m.insert(
            "o3".into(),
            ModelPricing {
                input_per_mtok: 10.0,
                output_per_mtok: 40.0,
                cache_write_per_mtok: None,
                cache_read_per_mtok: None,
            },
        );
        m.insert(
            "gpt-4.1".into(),
            ModelPricing {
                input_per_mtok: 2.0,
                output_per_mtok: 8.0,
                cache_write_per_mtok: None,
                cache_read_per_mtok: None,
            },
        );
        m.insert(
            "glm-5".into(),
            ModelPricing {
                input_per_mtok: 1.4,
                output_per_mtok: 4.4,
                cache_write_per_mtok: None,
                cache_read_per_mtok: Some(0.475),
            },
        );
        m.insert(
            "glm-5.1".into(),
            ModelPricing {
                input_per_mtok: 1.4,
                output_per_mtok: 4.4,
                cache_write_per_mtok: None,
                cache_read_per_mtok: Some(0.475),
            },
        );
        m
    }
}
