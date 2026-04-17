use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

use crate::model::{SessionSnapshot, SessionStatus};

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

// ============================================================
// Shared status inference
// ============================================================

/// Infer session status from the last activity timestamp.
///
/// Thresholds:
///   < 5 min  → Active
///   5–30 min → Idle
///   > 30 min → Done
///
/// If `process_alive` is true (e.g. Claude Code process still running),
/// the status is clamped to at least Idle — never Done while the process exists.
pub fn infer_status(last_activity: Option<DateTime<Utc>>, process_alive: bool) -> SessionStatus {
    let status = match last_activity {
        Some(ts) => {
            let age = Utc::now().signed_duration_since(ts);
            if age.num_minutes() < 5 {
                SessionStatus::Active
            } else if age.num_minutes() < 30 {
                SessionStatus::Idle
            } else {
                SessionStatus::Done
            }
        }
        None => SessionStatus::Done,
    };

    // Process-alive boost: at least Idle if process is still running
    if process_alive && status == SessionStatus::Done {
        SessionStatus::Idle
    } else {
        status
    }
}

pub mod claude_code;
pub mod code_buddy;
