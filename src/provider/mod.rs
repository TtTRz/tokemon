use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::model::SessionSnapshot;

/// Capabilities a provider declares — UI uses this to decide what to show.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub has_context_window: bool,
    pub has_reported_cost: bool,
    pub has_git_info: bool,
    pub has_cache_tokens: bool,
}

/// Events emitted by providers into the collector channel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ProviderEvent {
    /// Session state update
    Update(Box<SessionSnapshot>),
    /// Session ended
    Ended { session_id: String },
    /// Provider connection status change
    ConnectionStatus {
        provider: String,
        connected: bool,
        error: Option<String>,
    },
}

/// Trait every AI tool provider must implement.
#[allow(dead_code)]
#[async_trait]
pub trait Provider: Send + Sync {
    /// Full provider name (e.g. "Claude Code")
    fn name(&self) -> &str;

    /// Short label for session list (e.g. "CC", "CDX", "CB")
    fn short_label(&self) -> &str;

    /// Declare which fields this provider can populate
    fn capabilities(&self) -> ProviderCapabilities;

    /// Start collecting data. Implementations spawn internal tasks
    /// that push ProviderEvents into `tx`.
    async fn start(&self, tx: mpsc::Sender<ProviderEvent>) -> anyhow::Result<()>;

    /// Stop collecting and clean up.
    async fn stop(&self) -> anyhow::Result<()>;
}

pub mod claude_code;
