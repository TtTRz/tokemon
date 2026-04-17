use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use super::{Provider, ProviderCapabilities, ProviderEvent};
use crate::model::{SessionSnapshot, SessionStatus};

/// Claude Code provider — collects data from statusline socket + JSONL logs.
pub struct ClaudeCodeProvider {
    socket_path: PathBuf,
    log_dirs: Vec<PathBuf>,
}

impl ClaudeCodeProvider {
    pub fn new(socket_path: String, log_dirs: Vec<String>) -> Self {
        let expanded_socket = shellexpand::tilde(&socket_path).to_string();
        let log_dirs = log_dirs
            .into_iter()
            .map(|d| PathBuf::from(shellexpand::tilde(&d).to_string()))
            .collect();
        Self {
            socket_path: PathBuf::from(expanded_socket),
            log_dirs,
        }
    }
}

#[async_trait]
impl Provider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn short_label(&self) -> &str {
        "CC"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            has_context_window: true,
            has_reported_cost: true,
            has_git_info: true,
            has_cache_tokens: true,
        }
    }

    async fn start(&self, tx: mpsc::Sender<ProviderEvent>) -> anyhow::Result<()> {
        // 1. Scan JSONL logs for historical sessions
        for log_dir in &self.log_dirs {
            if !log_dir.exists() {
                tracing::warn!("Claude Code log dir not found: {}", log_dir.display());
                continue;
            }

            let snapshots = scan_all_sessions(log_dir)?;
            for snapshot in snapshots {
                let _ = tx.send(ProviderEvent::Update(Box::new(snapshot))).await;
            }

            // Watch for JSONL changes
            let tx_clone = tx.clone();
            let dir = log_dir.clone();
            tokio::spawn(async move {
                if let Err(e) = watch_logs(dir, tx_clone).await {
                    tracing::error!("Claude Code JSONL watcher error: {e}");
                }
            });
        }

        // 2. Listen on Unix socket for statusline real-time data
        let tx_socket = tx.clone();
        let sock_path = self.socket_path.clone();
        tokio::spawn(async move {
            if let Err(e) = listen_socket(sock_path, tx_socket).await {
                tracing::error!("Claude Code socket listener error: {e}");
            }
        });

        tracing::info!(
            "Claude Code provider started: {} log dirs, socket {}",
            self.log_dirs.len(),
            self.socket_path.display()
        );
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
        tracing::info!("Claude Code provider stopped");
        Ok(())
    }
}

// ============================================================
// Statusline socket listener
// ============================================================

/// JSON structure from Claude Code statusline (via stdin to the script).
#[derive(Debug, Deserialize)]
struct StatuslineData {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    model: Option<StatuslineModel>,
    #[serde(default)]
    context_window: Option<StatuslineContext>,
    #[serde(default)]
    cost: Option<StatuslineCost>,
    #[serde(default)]
    workspace: Option<StatuslineWorkspace>,
    #[serde(default)]
    cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatuslineModel {
    #[serde(default)]
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatuslineContext {
    #[serde(default)]
    total_input_tokens: Option<u64>,
    #[serde(default)]
    total_output_tokens: Option<u64>,
    #[serde(default)]
    context_window_size: Option<u64>,
    #[serde(default)]
    used_percentage: Option<f64>,
    #[serde(default)]
    current_usage: Option<StatuslineCurrentUsage>,
}

#[derive(Debug, Deserialize)]
struct StatuslineCurrentUsage {
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct StatuslineCost {
    #[serde(default)]
    total_cost_usd: Option<f64>,
    #[serde(default)]
    total_duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct StatuslineWorkspace {
    #[serde(default)]
    current_dir: Option<String>,
}

fn statusline_to_snapshot(data: &StatuslineData) -> Option<SessionSnapshot> {
    let session_id = data.session_id.as_ref()?;
    let model = data
        .model
        .as_ref()
        .and_then(|m| m.id.clone())
        .unwrap_or_default();

    let ctx = data.context_window.as_ref();
    let input_tokens = ctx.and_then(|c| c.total_input_tokens).unwrap_or(0);
    let output_tokens = ctx.and_then(|c| c.total_output_tokens).unwrap_or(0);
    let context_max = ctx.and_then(|c| c.context_window_size);
    let context_pct = ctx.and_then(|c| c.used_percentage);
    let context_used =
        context_max.map(|max| (context_pct.unwrap_or(0.0) / 100.0 * max as f64) as u64);

    let cache_creation = ctx
        .and_then(|c| c.current_usage.as_ref())
        .map(|u| u.cache_creation_input_tokens)
        .unwrap_or(0);
    let cache_read = ctx
        .and_then(|c| c.current_usage.as_ref())
        .map(|u| u.cache_read_input_tokens)
        .unwrap_or(0);

    let cost_reported = data.cost.as_ref().and_then(|c| c.total_cost_usd);
    let duration_ms = data
        .cost
        .as_ref()
        .and_then(|c| c.total_duration_ms)
        .unwrap_or(0);
    let total_secs = duration_ms as f64 / 1000.0;
    let input_tps = if total_secs > 0.0 {
        Some(input_tokens as f64 / total_secs)
    } else {
        None
    };
    let output_tps = if total_secs > 0.0 {
        Some(output_tokens as f64 / total_secs)
    } else {
        None
    };

    let work_dir = data
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| data.cwd.clone());

    Some(SessionSnapshot {
        session_id: session_id.clone(),
        provider: "Claude Code".into(),
        model,
        input_tokens,
        output_tokens,
        cache_creation_tokens: cache_creation,
        cache_read_tokens: cache_read,
        context_tokens: context_used,
        context_max,
        context_window_pct: context_pct,
        input_tps,
        output_tps,
        cost_reported,
        git_branch: None, // statusline doesn't include git branch
        work_dir,
        status: SessionStatus::Active, // If statusline is sending data, it's active
        timestamp: Utc::now(),
        subagent_count: 0,
    })
}

/// Listen on a Unix domain socket for statusline JSON data.
async fn listen_socket(path: PathBuf, tx: mpsc::Sender<ProviderEvent>) -> anyhow::Result<()> {
    // Remove stale socket
    let _ = std::fs::remove_file(&path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&path)?;
    tracing::info!("Statusline socket listening on {}", path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let reader = tokio::io::BufReader::new(stream);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if line.trim().is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<StatuslineData>(&line) {
                            Ok(data) => {
                                if let Some(snapshot) = statusline_to_snapshot(&data) {
                                    let _ =
                                        tx.send(ProviderEvent::Update(Box::new(snapshot))).await;
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Failed to parse statusline JSON: {e}");
                            }
                        }
                    }
                });
            }
            Err(e) => {
                tracing::warn!("Socket accept error: {e}");
            }
        }
    }
}

// ============================================================
// JSONL log parsing (historical data)
// ============================================================

#[derive(Debug, Deserialize)]
struct LogEntry {
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default, rename = "sessionId")]
    session_id: Option<String>,
    #[serde(default)]
    message: Option<AssistantMessage>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default, rename = "gitBranch")]
    git_branch: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default, rename = "durationMs")]
    duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AssistantMessage {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<LogUsage>,
}

#[derive(Debug, Deserialize)]
struct LogUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[derive(Debug, Clone)]
struct SessionAccumulator {
    session_id: String,
    model: String,
    total_input: u64,
    total_output: u64,
    total_cache_creation: u64,
    total_cache_read: u64,
    git_branch: Option<String>,
    cwd: Option<String>,
    last_timestamp: Option<DateTime<Utc>>,
    turn_count: u64,
    total_duration_ms: u64,
    subagent_count: usize,
    /// Per-turn estimated cost accumulator (more accurate than post-hoc calculation)
    per_turn_cost: f64,
    /// Track the last model seen, for per-turn cost calculation
    last_model: String,
}

impl SessionAccumulator {
    fn new(session_id: String) -> Self {
        Self {
            session_id,
            model: String::new(),
            total_input: 0,
            total_output: 0,
            total_cache_creation: 0,
            total_cache_read: 0,
            git_branch: None,
            cwd: None,
            last_timestamp: None,
            turn_count: 0,
            total_duration_ms: 0,
            subagent_count: 0,
            per_turn_cost: 0.0,
            last_model: String::new(),
        }
    }

    fn apply(&mut self, entry: &LogEntry) {
        if let Some(ref branch) = entry.git_branch {
            self.git_branch = Some(branch.clone());
        }
        if let Some(ref cwd) = entry.cwd {
            self.cwd = Some(cwd.clone());
        }
        if let Some(dt) = entry
            .timestamp
            .as_ref()
            .and_then(|ts| ts.parse::<DateTime<Utc>>().ok())
        {
            self.last_timestamp = Some(dt);
        }

        if let Some(msg) = entry
            .message
            .as_ref()
            .filter(|_| entry.entry_type == "assistant")
        {
            if let Some(ref model) = msg.model {
                self.model.clone_from(model);
                self.last_model.clone_from(model);
            }
            if let Some(ref usage) = msg.usage {
                self.total_input += usage.input_tokens;
                self.total_output += usage.output_tokens;
                self.total_cache_creation += usage.cache_creation_input_tokens;
                self.total_cache_read += usage.cache_read_input_tokens;

                // Per-turn cost: calculate cost for this single API call
                self.per_turn_cost += crate::pricing::estimate_turn_cost_builtin(
                    &self.last_model,
                    usage.input_tokens,
                    usage.output_tokens,
                    usage.cache_creation_input_tokens,
                    usage.cache_read_input_tokens,
                );
            }
        }

        if let Some(ms) = entry.duration_ms.filter(|_| {
            entry.entry_type == "system" && entry.subtype.as_deref() == Some("turn_duration")
        }) {
            self.turn_count += 1;
            self.total_duration_ms += ms;
        }
    }

    fn to_snapshot(&self) -> SessionSnapshot {
        let total_secs = self.total_duration_ms as f64 / 1000.0;
        let input_tps = if total_secs > 0.0 {
            Some(self.total_input as f64 / total_secs)
        } else {
            None
        };
        let output_tps = if total_secs > 0.0 {
            Some(self.total_output as f64 / total_secs)
        } else {
            None
        };

        // Status: use shared inference + process-alive check
        let process_alive = is_claude_session_alive(&self.session_id);
        let status = super::infer_status(self.last_timestamp, process_alive);
        let now = Utc::now();

        SessionSnapshot {
            session_id: self.session_id.clone(),
            provider: "Claude Code".into(),
            model: self.model.clone(),
            input_tokens: self.total_input,
            output_tokens: self.total_output,
            cache_creation_tokens: self.total_cache_creation,
            cache_read_tokens: self.total_cache_read,
            context_tokens: None,
            context_max: None,
            context_window_pct: None,
            input_tps,
            output_tps,
            // Use per-turn accumulated cost (more accurate than post-hoc estimate)
            cost_reported: if self.per_turn_cost > 0.0 {
                Some(self.per_turn_cost)
            } else {
                None
            },
            git_branch: self.git_branch.clone(),
            work_dir: self.cwd.clone(),
            status,
            timestamp: self.last_timestamp.unwrap_or(now),
            subagent_count: self.subagent_count,
        }
    }
}

fn parse_session_file(path: &Path) -> anyhow::Result<Option<SessionSnapshot>> {
    let content = std::fs::read_to_string(path)?;
    let mut acc: Option<SessionAccumulator> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: LogEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if let Some(ref sid) = entry.session_id {
            let accumulator = acc.get_or_insert_with(|| SessionAccumulator::new(sid.clone()));
            accumulator.apply(&entry);
        } else if let Some(ref accumulator) = acc {
            let mut a = accumulator.clone();
            a.apply(&entry);
            acc = Some(a);
        }
    }

    // Merge subagent tokens into this session
    if let Some(ref mut accumulator) = acc {
        let session_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let subagents_dir = path
            .parent()
            .unwrap_or(path)
            .join(session_id)
            .join("subagents");
        if subagents_dir.is_dir() {
            merge_subagent_tokens(accumulator, &subagents_dir);
        }
    }

    Ok(acc.filter(|a| !a.model.is_empty()).map(|a| a.to_snapshot()))
}

/// Scan subagent JSONL files and add their token usage to the parent accumulator.
fn merge_subagent_tokens(acc: &mut SessionAccumulator, subagents_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(subagents_dir) else {
        return;
    };

    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            count += 1;
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let entry: LogEntry = match serde_json::from_str(line) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                // Only accumulate token usage from assistant messages
                if entry.entry_type == "assistant"
                    && let Some(ref msg) = entry.message
                    && let Some(ref usage) = msg.usage
                {
                    acc.total_input += usage.input_tokens;
                    acc.total_output += usage.output_tokens;
                    acc.total_cache_creation += usage.cache_creation_input_tokens;
                    acc.total_cache_read += usage.cache_read_input_tokens;
                    // Per-turn cost for subagent
                    let model = msg.model.as_deref().unwrap_or(&acc.last_model);
                    acc.per_turn_cost += crate::pricing::estimate_turn_cost_builtin(
                        model,
                        usage.input_tokens,
                        usage.output_tokens,
                        usage.cache_creation_input_tokens,
                        usage.cache_read_input_tokens,
                    );
                }
                // Update last_timestamp if subagent is more recent
                if let Some(dt) = entry
                    .timestamp
                    .as_ref()
                    .and_then(|ts| ts.parse::<DateTime<Utc>>().ok())
                    && acc.last_timestamp.is_none_or(|prev| dt > prev)
                {
                    acc.last_timestamp = Some(dt);
                }
            }
        }
    }
    acc.subagent_count = count;
}

fn scan_all_sessions(base_dir: &Path) -> anyhow::Result<Vec<SessionSnapshot>> {
    let mut snapshots = Vec::new();

    if let Ok(projects) = std::fs::read_dir(base_dir) {
        for project_entry in projects.flatten() {
            let project_path = project_entry.path();
            if !project_path.is_dir() {
                continue;
            }
            if let Ok(files) = std::fs::read_dir(&project_path) {
                for file_entry in files.flatten() {
                    let file_path = file_entry.path();
                    if file_path.extension().is_some_and(|e| e == "jsonl") {
                        match parse_session_file(&file_path) {
                            Ok(Some(snapshot)) => snapshots.push(snapshot),
                            Ok(None) => {}
                            Err(e) => {
                                tracing::warn!("Failed to parse {}: {e}", file_path.display());
                            }
                        }
                    }
                }
            }
        }
    }

    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    tracing::info!(
        "Scanned {} sessions from {}",
        snapshots.len(),
        base_dir.display()
    );
    Ok(snapshots)
}

async fn watch_logs(dir: PathBuf, tx: mpsc::Sender<ProviderEvent>) -> anyhow::Result<()> {
    let (notify_tx, mut notify_rx) = mpsc::channel::<PathBuf>(256);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            let Ok(event) = res else { return };
            if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                return;
            }
            for path in event.paths {
                if path.extension().is_some_and(|e| e == "jsonl") {
                    let _ = notify_tx.blocking_send(path);
                }
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&dir, RecursiveMode::Recursive)?;
    tracing::info!("Watching for JSONL changes in {}", dir.display());

    let mut last_parsed: HashMap<PathBuf, std::time::Instant> = HashMap::new();
    let debounce = std::time::Duration::from_millis(500);

    loop {
        if let Some(path) = notify_rx.recv().await {
            let now = std::time::Instant::now();

            // If this is a subagent JSONL, resolve to the parent session file
            let parse_path = resolve_to_parent_session(&path);

            if last_parsed
                .get(&parse_path)
                .is_some_and(|&last| now.duration_since(last) < debounce)
            {
                continue;
            }
            last_parsed.insert(parse_path.clone(), now);

            match parse_session_file(&parse_path) {
                Ok(Some(snapshot)) => {
                    let _ = tx.send(ProviderEvent::Update(Box::new(snapshot))).await;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("Failed to re-parse {}: {e}", path.display());
                }
            }
        }
    }
}

/// If a JSONL path is inside a `subagents/` directory, resolve it to the parent session JSONL.
/// e.g. `.../projects/foo/SESSION_ID/subagents/agent-xxx.jsonl` → `.../projects/foo/SESSION_ID.jsonl`
fn resolve_to_parent_session(path: &Path) -> PathBuf {
    if let Some(parent) = path.parent()
        && parent.file_name().is_some_and(|n| n == "subagents")
        && let Some(session_dir) = parent.parent()
        && let Some(session_id) = session_dir.file_name()
        && let Some(project_dir) = session_dir.parent()
    {
        let parent_jsonl = project_dir.join(format!("{}.jsonl", session_id.to_string_lossy()));
        if parent_jsonl.exists() {
            return parent_jsonl;
        }
    }
    path.to_path_buf()
}

/// Check if a Claude Code session is still alive by looking for its tmp directory.
///
/// Claude Code creates `/tmp/claude-{uid}/{project}/{session_id}/` while running.
/// The directory is cleaned up when the session exits.
fn is_claude_session_alive(session_id: &str) -> bool {
    // SAFETY: getuid() is a POSIX syscall that always succeeds, has no side effects,
    // and requires no preconditions. The unsafe block is only needed for FFI boundary.
    let uid = unsafe { libc::getuid() };
    let tmp = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp/".into());
    let claude_tmp = PathBuf::from(tmp).join(format!("claude-{uid}"));

    if !claude_tmp.exists() {
        return false;
    }

    // Scan project dirs for a session_id match
    let Ok(projects) = std::fs::read_dir(&claude_tmp) else {
        return false;
    };

    for entry in projects.flatten() {
        let session_dir = entry.path().join(session_id);
        if session_dir.is_dir() {
            return true;
        }
    }

    false
}
