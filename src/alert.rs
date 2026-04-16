use crate::config::AlertsConfig;
use crate::model::{Alert, SessionSnapshot};
use crate::pricing::PricingEngine;

/// Evaluates alert rules against session snapshots.
pub struct AlertEngine {
    config: AlertsConfig,
}

impl AlertEngine {
    pub fn new(config: &AlertsConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check a snapshot and return any triggered alerts.
    pub fn check(&self, snapshot: &SessionSnapshot, pricing: &PricingEngine) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Context window check
        if let Some(pct) = snapshot
            .context_window_pct
            .filter(|&p| p >= self.config.context_warn_pct)
        {
            alerts.push(Alert::ContextHigh {
                session_id: snapshot.session_id.clone(),
                provider: snapshot.provider.clone(),
                pct,
            });
        }

        // Cost threshold check
        let cost = snapshot
            .cost_reported
            .unwrap_or_else(|| pricing.estimate_cost(snapshot));
        if cost >= self.config.cost_threshold_usd {
            alerts.push(Alert::CostThreshold {
                session_id: snapshot.session_id.clone(),
                cost,
                threshold: self.config.cost_threshold_usd,
            });
        }

        alerts
    }
}
