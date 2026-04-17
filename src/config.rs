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
    #[serde(default)]
    pub locale: Option<String>,
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
            locale: None,
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
        let mut models = HashMap::new();
        models.insert(
            "claude-sonnet-4".into(),
            ModelPricingConfig {
                input: 3.0,
                output: 15.0,
                cache_write: Some(3.75),
                cache_read: Some(0.30),
            },
        );
        models.insert(
            "claude-4.6-opus".into(),
            ModelPricingConfig {
                input: 5.0,
                output: 25.0,
                cache_write: Some(6.25),
                cache_read: Some(0.50),
            },
        );
        models.insert(
            "claude-haiku-3.5".into(),
            ModelPricingConfig {
                input: 0.80,
                output: 4.0,
                cache_write: Some(1.0),
                cache_read: Some(0.08),
            },
        );
        models.insert(
            "o3".into(),
            ModelPricingConfig {
                input: 10.0,
                output: 40.0,
                cache_write: None,
                cache_read: None,
            },
        );
        models.insert(
            "gpt-4.1".into(),
            ModelPricingConfig {
                input: 2.0,
                output: 8.0,
                cache_write: None,
                cache_read: None,
            },
        );
        models.insert(
            "glm-5".into(),
            ModelPricingConfig {
                input: 1.4,
                output: 4.4,
                cache_write: None,
                cache_read: Some(0.475),
            },
        );
        models.insert(
            "glm-5.1".into(),
            ModelPricingConfig {
                input: 1.4,
                output: 4.4,
                cache_write: None,
                cache_read: Some(0.475),
            },
        );
        Self {
            default_input: 3.0,
            default_output: 15.0,
            models,
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
    /// Load from `~/.config/tokemon/config.toml`.
    /// If the file doesn't exist, generate a default config and write it to disk.
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
            // Generate default config file
            let config = Self::default();
            config.write_default(&path);
        }

        Self::default()
    }

    /// Write a commented default config to disk.
    fn write_default(&self, path: &std::path::Path) {
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            tracing::warn!("Failed to create config dir: {e}");
            return;
        }

        let content = Self::default_config_toml();
        match std::fs::write(path, content) {
            Ok(()) => tracing::info!("Generated default config at {}", path.display()),
            Err(e) => tracing::warn!("Failed to write default config: {e}"),
        }
    }

    /// Generate a human-readable default config with comments.
    fn default_config_toml() -> String {
        let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp/".into());
        let mut s = String::new();

        s.push_str("# tokemon — Token Monitor configuration\n");
        s.push_str("# https://github.com/TtTRz/tokemon\n\n");

        s.push_str("[general]\n");
        s.push_str("tick_rate_ms = 250\n");
        s.push_str("theme = \"dark\"\n");
        s.push_str("# locale = \"zh-CN\"  # Display language (auto-detected if omitted)\n\n");

        s.push_str("[providers.claude_code]\n");
        s.push_str("enabled = true\n");
        s.push_str(&format!(
            "socket_path = \"{}tokemon-claude.sock\"\n",
            tmpdir
        ));
        s.push_str("log_dirs = [\"~/.claude/projects/\"]\n\n");

        s.push_str("[providers.code_buddy]\n");
        s.push_str("enabled = true\n");
        s.push_str(&format!(
            "socket_path = \"{}tokemon-codebuddy.sock\"\n",
            tmpdir
        ));
        s.push_str("log_dirs = [\"~/.codebuddy/projects/\"]\n\n");

        s.push_str("# Pricing: $/1M tokens. Used for cost estimation when provider\n");
        s.push_str("# doesn't report actual cost. Prefix-matched against model names.\n");
        s.push_str("[pricing]\n");
        s.push_str("default_input = 3.0\n");
        s.push_str("default_output = 15.0\n\n");

        // Sort model names for stable output
        let defaults = PricingConfig::default();
        let mut models: Vec<_> = defaults.models.iter().collect();
        models.sort_by_key(|(k, _)| (*k).clone());

        for (name, p) in &models {
            s.push_str(&format!("[pricing.models.\"{}\"]\n", name));
            s.push_str(&format!("input = {}\n", p.input));
            s.push_str(&format!("output = {}\n", p.output));
            if let Some(cw) = p.cache_write {
                s.push_str(&format!("cache_write = {}\n", cw));
            }
            if let Some(cr) = p.cache_read {
                s.push_str(&format!("cache_read = {}\n", cr));
            }
            s.push('\n');
        }

        s.push_str("[alerts]\n");
        s.push_str("context_warn_pct = 80.0\n");
        s.push_str("context_crit_pct = 95.0\n");
        s.push_str("cost_threshold_usd = 5.0\n");

        s
    }
}
