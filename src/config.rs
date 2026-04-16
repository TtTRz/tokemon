use std::collections::HashMap;

use serde::Deserialize;

/// Top-level configuration loaded from `~/.config/tokemon/config.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub providers: ProvidersConfig,
    pub pricing: PricingConfig,
    pub alerts: AlertsConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub tick_rate_ms: u64,
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProvidersConfig {
    pub claude_code: ClaudeCodeConfig,
    pub code_buddy: CodeBuddyConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClaudeCodeConfig {
    pub enabled: bool,
    pub socket_path: String,
    pub log_dirs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CodeBuddyConfig {
    pub enabled: bool,
    pub socket_path: String,
    pub log_dirs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PricingConfig {
    pub default_input: f64,
    pub default_output: f64,
    #[serde(default)]
    pub models: HashMap<String, ModelPricingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricingConfig {
    pub input: f64,
    pub output: f64,
    pub cache_write: Option<f64>,
    pub cache_read: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlertsConfig {
    pub context_warn_pct: f64,
    pub context_crit_pct: f64,
    pub cost_threshold_usd: f64,
}

// --- Defaults ---

// Config: all fields have their own Default, so derive works.
#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            providers: ProvidersConfig::default(),
            pricing: PricingConfig::default(),
            alerts: AlertsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            theme: "dark".into(),
        }
    }
}

// ProvidersConfig: single field with its own Default.
#[allow(clippy::derivable_impls)]
impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            claude_code: ClaudeCodeConfig::default(),
            code_buddy: CodeBuddyConfig::default(),
        }
    }
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
        Self {
            enabled: true,
            socket_path: format!("{tmpdir}tokemon-claude.sock"),
            log_dirs: vec!["~/.claude/projects/".into()],
        }
    }
}

impl Default for CodeBuddyConfig {
    fn default() -> Self {
        let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
        Self {
            enabled: true,
            socket_path: format!("{tmpdir}tokemon-codebuddy.sock"),
            log_dirs: vec!["~/.codebuddy/projects/".into()],
        }
    }
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            default_input: 3.0,
            default_output: 15.0,
            models: HashMap::new(),
        }
    }
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            context_warn_pct: 80.0,
            context_crit_pct: 95.0,
            cost_threshold_usd: 5.0,
        }
    }
}

impl Config {
    /// Load from `~/.config/tokemon/config.toml`, falling back to defaults.
    pub fn load() -> Self {
        let path = dirs::config_dir()
            .map(|d| d.join("tokemon").join("config.toml"))
            .unwrap_or_default();

        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        tracing::info!("Loaded config from {}", path.display());
                        return cfg;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse config: {e}, using defaults");
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config: {e}, using defaults");
                }
            }
        } else {
            tracing::info!("No config file found at {}, using defaults", path.display());
        }

        Self::default()
    }
}
