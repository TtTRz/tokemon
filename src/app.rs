use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;

use crate::alert::AlertEngine;
use crate::model::{Alert, SessionSnapshot, SessionStatus};
use crate::pricing::PricingEngine;
use crate::provider::ProviderEvent;

/// Which tab is active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveTab {
    /// Overview: active + idle sessions
    Overview,
    /// History: done sessions
    History,
    /// Individual session detail, by session_id
    Session(String),
}

pub struct App {
    pub should_quit: bool,
    pub show_help: bool,

    /// Current active tab
    pub active_tab: ActiveTab,

    /// All tracked sessions
    pub sessions: Vec<SessionSnapshot>,
    /// Selected row in overview table
    pub overview_selected: usize,
    /// Number of columns in overview grid (updated by UI on render)
    pub overview_cols: usize,
    /// Scroll offset in overview (number of card rows scrolled)
    pub overview_scroll: usize,
    /// How many card rows fit in the viewport (set by render)
    pub overview_visible_rows: usize,

    /// Selected row in history tab
    pub history_selected: usize,
    pub history_scroll: usize,
    pub history_visible_rows: usize,

    /// Per-session time-series: session_id -> Vec<(elapsed_secs, tokens)>
    pub session_token_series: std::collections::HashMap<String, Vec<(f64, f64)>>,
    /// Per-session cost series: session_id -> Vec<(elapsed_secs, cost)>
    pub session_cost_series: std::collections::HashMap<String, Vec<(f64, f64)>>,
    /// Per-session start times (first seen timestamp)
    session_start_times: std::collections::HashMap<String, DateTime<Utc>>,

    /// Global aggregate series (for overview), X = elapsed since app start
    pub total_token_series: Vec<(f64, f64)>,
    pub total_cost_series: Vec<(f64, f64)>,
    /// App start time
    app_start: DateTime<Utc>,

    /// Active alerts
    pub alerts: Vec<Alert>,

    pub pricing: PricingEngine,
    pub alert_engine: AlertEngine,

    max_series_points: usize,
}

impl App {
    pub fn new(pricing: PricingEngine, alert_engine: AlertEngine) -> Self {
        Self {
            should_quit: false,
            show_help: false,
            active_tab: ActiveTab::Overview,
            sessions: Vec::new(),
            overview_selected: 0,
            overview_cols: 2,
            overview_scroll: 0,
            overview_visible_rows: 2,
            history_selected: 0,
            history_scroll: 0,
            history_visible_rows: 2,
            session_token_series: std::collections::HashMap::new(),
            session_cost_series: std::collections::HashMap::new(),
            session_start_times: std::collections::HashMap::new(),
            total_token_series: Vec::new(),
            total_cost_series: Vec::new(),
            app_start: Utc::now(),
            alerts: Vec::new(),
            pricing,
            alert_engine,
            max_series_points: 300,
        }
    }

    /// Sessions that are Active or Idle (shown in Overview + as tabs).
    pub fn live_sessions(&self) -> Vec<&SessionSnapshot> {
        self.sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Active | SessionStatus::Idle))
            .collect()
    }

    /// Sessions that are Done (shown in History tab).
    pub fn done_sessions(&self) -> Vec<&SessionSnapshot> {
        self.sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Done | SessionStatus::Disconnected))
            .collect()
    }

    /// Ordered list of tab labels: Overview + History + live sessions.
    pub fn tab_labels(&self) -> Vec<String> {
        let mut labels = vec!["Overview".into(), "History".into()];
        for s in self.live_sessions() {
            let id_short = &s.session_id[..s.session_id.len().min(6)];
            labels.push(format!("{} #{id_short}", s.provider));
        }
        labels
    }

    /// Index of the currently active tab (0=Overview, 1=History, 2+=live sessions).
    pub fn active_tab_index(&self) -> usize {
        match &self.active_tab {
            ActiveTab::Overview => 0,
            ActiveTab::History => 1,
            ActiveTab::Session(id) => self
                .live_sessions()
                .iter()
                .position(|s| &s.session_id == id)
                .map(|i| i + 2)
                .unwrap_or(0),
        }
    }

    pub fn handle_terminal_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if self.show_help {
                self.show_help = false;
                return;
            }

            match (key.code, key.modifiers) {
                // Quit
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                // Help
                (KeyCode::Char('?'), _) => {
                    self.show_help = true;
                }
                // Number keys: 1=Overview, 2..9=session tabs
                (KeyCode::Char(c @ '1'..='9'), _) => {
                    let idx = (c as u32 - '1' as u32) as usize;
                    self.set_tab_by_index(idx);
                }
                // Tab navigation: Tab/Shift+Tab
                (KeyCode::Tab, KeyModifiers::NONE) => {
                    self.next_tab();
                }
                (KeyCode::BackTab, _) => {
                    self.prev_tab();
                }
                // Within Overview/History: j/k/h/l + arrows to navigate grid
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => match self.active_tab {
                    ActiveTab::Overview => self.overview_move_down(),
                    ActiveTab::History => self.history_move_down(),
                    _ => {}
                },
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => match self.active_tab {
                    ActiveTab::Overview => self.overview_move_up(),
                    ActiveTab::History => self.history_move_up(),
                    _ => {}
                },
                (KeyCode::Char('l'), _) | (KeyCode::Right, _) => {
                    if self.active_tab == ActiveTab::Overview {
                        self.overview_move_right();
                    }
                }
                (KeyCode::Char('h'), _) | (KeyCode::Left, _) => {
                    if self.active_tab == ActiveTab::Overview {
                        self.overview_move_left();
                    }
                }
                (KeyCode::Enter, _) => match self.active_tab {
                    ActiveTab::Overview => self.jump_to_selected_session(),
                    ActiveTab::History => self.jump_to_selected_history(),
                    _ => {}
                },
                // Esc: back to Overview, or quit if already on Overview
                (KeyCode::Esc, _) => {
                    if self.active_tab == ActiveTab::Overview {
                        self.should_quit = true;
                    } else {
                        self.active_tab = ActiveTab::Overview;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_provider_event(&mut self, event: ProviderEvent) {
        match event {
            ProviderEvent::Update(snapshot) => {
                let sid = snapshot.session_id.clone();
                let now = Utc::now();

                // Per-session: X = seconds since first seen this session
                let is_new_session = !self.session_start_times.contains_key(&sid);
                let session_start = *self
                    .session_start_times
                    .entry(sid.clone())
                    .or_insert(snapshot.timestamp);
                let session_elapsed = now
                    .signed_duration_since(session_start)
                    .num_seconds()
                    .max(0) as f64;

                let total_tokens = (snapshot.input_tokens + snapshot.output_tokens) as f64;
                let tok_series = self.session_token_series.entry(sid.clone()).or_default();
                // Seed origin point so chart doesn't draw from (0,0)
                if is_new_session && tok_series.is_empty() && session_elapsed > 0.0 {
                    tok_series.push((0.0, total_tokens));
                }
                tok_series.push((session_elapsed, total_tokens));
                Self::trim_series(tok_series, self.max_series_points);

                let cost = snapshot
                    .cost_reported
                    .unwrap_or_else(|| self.pricing.estimate_cost(&snapshot));
                let cost_series = self.session_cost_series.entry(sid.clone()).or_default();
                if is_new_session && cost_series.is_empty() && session_elapsed > 0.0 {
                    cost_series.push((0.0, cost));
                }
                cost_series.push((session_elapsed, cost));
                Self::trim_series(cost_series, self.max_series_points);

                // Alerts (before upsert consumes snapshot)
                let new_alerts = self.alert_engine.check(&snapshot, &self.pricing);
                if !new_alerts.is_empty() {
                    self.alerts = new_alerts;
                }

                // Upsert session
                if let Some(existing) = self.sessions.iter_mut().find(|s| s.session_id == sid) {
                    *existing = *snapshot;
                } else {
                    self.sessions.push(*snapshot);
                }

                // Global aggregate: X = seconds since app start
                let app_elapsed = now
                    .signed_duration_since(self.app_start)
                    .num_seconds()
                    .max(0) as f64;

                let total_all_tokens: f64 = self
                    .sessions
                    .iter()
                    .map(|s| (s.input_tokens + s.output_tokens) as f64)
                    .sum();
                self.total_token_series
                    .push((app_elapsed, total_all_tokens));
                Self::trim_series(&mut self.total_token_series, self.max_series_points);

                let total_all_cost: f64 = self
                    .sessions
                    .iter()
                    .map(|s| {
                        s.cost_reported
                            .unwrap_or_else(|| self.pricing.estimate_cost(s))
                    })
                    .sum();
                self.total_cost_series.push((app_elapsed, total_all_cost));
                Self::trim_series(&mut self.total_cost_series, self.max_series_points);
            }
            ProviderEvent::Ended { session_id } => {
                if let Some(s) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.session_id == session_id)
                {
                    s.status = crate::model::SessionStatus::Done;
                }
            }
            ProviderEvent::ConnectionStatus {
                provider,
                connected: false,
                error,
            } => {
                self.alerts
                    .push(Alert::ProviderDisconnected { provider, error });
            }
            ProviderEvent::ConnectionStatus {
                connected: true, ..
            } => {}
        }
    }

    pub fn check_alerts(&mut self) {
        let mut all_alerts = Vec::new();
        for session in &self.sessions {
            all_alerts.extend(self.alert_engine.check(session, &self.pricing));
        }
        if !all_alerts.is_empty() {
            self.alerts = all_alerts;
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        crate::ui::render(frame, self);
        if self.show_help {
            crate::ui::help::render(frame, frame.area());
        }
    }

    // --- Tab navigation ---

    fn next_tab(&mut self) {
        let live = self.live_sessions();
        let total = 2 + live.len(); // Overview + History + live sessions
        let cur = self.active_tab_index();
        let next = (cur + 1) % total;
        self.set_tab_by_index(next);
    }

    fn prev_tab(&mut self) {
        let live = self.live_sessions();
        let total = 2 + live.len();
        let cur = self.active_tab_index();
        let prev = if cur == 0 { total - 1 } else { cur - 1 };
        self.set_tab_by_index(prev);
    }

    fn set_tab_by_index(&mut self, idx: usize) {
        match idx {
            0 => self.active_tab = ActiveTab::Overview,
            1 => self.active_tab = ActiveTab::History,
            _ => {
                let live = self.live_sessions();
                if let Some(s) = live.get(idx - 2) {
                    self.active_tab = ActiveTab::Session(s.session_id.clone());
                }
            }
        }
    }

    // --- Overview grid navigation ---

    fn overview_move_down(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let next = self.overview_selected + self.overview_cols;
        if next < self.sessions.len() {
            self.overview_selected = next;
        }
        self.overview_ensure_visible();
    }

    fn overview_move_up(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        self.overview_selected = self.overview_selected.saturating_sub(self.overview_cols);
        self.overview_ensure_visible();
    }

    fn overview_move_right(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let col = self.overview_selected % self.overview_cols;
        if col + 1 < self.overview_cols {
            let next = self.overview_selected + 1;
            if next < self.sessions.len() {
                self.overview_selected = next;
            }
        }
    }

    fn overview_move_left(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let col = self.overview_selected % self.overview_cols;
        if col > 0 {
            self.overview_selected -= 1;
        }
    }

    /// Adjust scroll so the selected card row is visible.
    /// Called by the UI after setting `overview_visible_rows`.
    pub fn overview_ensure_visible(&mut self) {
        if self.overview_cols == 0 {
            return;
        }
        let selected_row = self.overview_selected / self.overview_cols;
        // Scroll up if above viewport
        if selected_row < self.overview_scroll {
            self.overview_scroll = selected_row;
        }
        // Scroll down if below viewport — visible_rows set by render
        if selected_row >= self.overview_scroll + self.overview_visible_rows.max(1) {
            self.overview_scroll = selected_row - self.overview_visible_rows.max(1) + 1;
        }
    }

    // --- History list navigation ---

    fn history_move_down(&mut self) {
        let count = self.done_sessions().len();
        if count > 0 && self.history_selected + 1 < count {
            self.history_selected += 1;
            self.history_ensure_visible();
        }
    }

    fn history_move_up(&mut self) {
        self.history_selected = self.history_selected.saturating_sub(1);
        self.history_ensure_visible();
    }

    pub fn history_ensure_visible(&mut self) {
        if self.history_selected < self.history_scroll {
            self.history_scroll = self.history_selected;
        }
        if self.history_selected >= self.history_scroll + self.history_visible_rows.max(1) {
            self.history_scroll = self.history_selected - self.history_visible_rows.max(1) + 1;
        }
    }

    fn jump_to_selected_session(&mut self) {
        let live = self.live_sessions();
        if let Some(s) = live.get(self.overview_selected) {
            self.active_tab = ActiveTab::Session(s.session_id.clone());
        }
    }

    fn jump_to_selected_history(&mut self) {
        let done = self.done_sessions();
        if let Some(s) = done.get(self.history_selected) {
            self.active_tab = ActiveTab::Session(s.session_id.clone());
        }
    }

    fn trim_series(series: &mut Vec<(f64, f64)>, max: usize) {
        while series.len() > max {
            series.remove(0);
        }
    }

    /// Add demo sessions for development/testing.
    pub fn add_demo_sessions(&mut self) {
        use crate::model::{SessionSnapshot, SessionStatus};
        use chrono::Utc;

        let demos = vec![
            SessionSnapshot {
                session_id: "a1b2c3".into(),
                provider: "Claude Code".into(),
                model: "claude-opus-4-20250514".into(),
                input_tokens: 12_600_000,
                output_tokens: 109_900,
                cache_creation_tokens: 30_000,
                cache_read_tokens: 12_000_000,
                context_tokens: Some(258_300),
                context_max: Some(1_000_000),
                context_window_pct: Some(25.8),
                input_tps: Some(6100.0),
                output_tps: Some(28.0),
                cost_reported: Some(1.23),
                git_branch: Some("feat/auth".into()),
                work_dir: Some("~/projects/myapp".into()),
                status: SessionStatus::Active,
                timestamp: Utc::now(),
            },
            SessionSnapshot {
                session_id: "d4e5f6".into(),
                provider: "Claude Code".into(),
                model: "claude-sonnet-4-20250514".into(),
                input_tokens: 3_200_000,
                output_tokens: 420_000,
                cache_creation_tokens: 0,
                cache_read_tokens: 2_800_000,
                context_tokens: Some(165_000),
                context_max: Some(200_000),
                context_window_pct: Some(82.5),
                input_tps: Some(3200.0),
                output_tps: Some(45.5),
                cost_reported: Some(0.45),
                git_branch: Some("main".into()),
                work_dir: Some("~/projects/api".into()),
                status: SessionStatus::Idle,
                timestamp: Utc::now(),
            },
            SessionSnapshot {
                session_id: "g7h8i9".into(),
                provider: "Codex".into(),
                model: "o3".into(),
                input_tokens: 5_000_000,
                output_tokens: 800_000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                context_tokens: None,
                context_max: None,
                context_window_pct: None,
                input_tps: Some(5000.0),
                output_tps: Some(120.0),
                cost_reported: None,
                git_branch: Some("fix/bug-123".into()),
                work_dir: Some("~/projects/backend".into()),
                status: SessionStatus::Active,
                timestamp: Utc::now(),
            },
            SessionSnapshot {
                session_id: "j0k1l2".into(),
                provider: "CodeBuddy".into(),
                model: "gpt-4.1".into(),
                input_tokens: 800_000,
                output_tokens: 95_000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                context_tokens: Some(15_000),
                context_max: Some(128_000),
                context_window_pct: Some(11.7),
                input_tps: Some(800.0),
                output_tps: Some(15.0),
                cost_reported: None,
                git_branch: None,
                work_dir: Some("~/projects/frontend".into()),
                status: SessionStatus::Done,
                timestamp: Utc::now(),
            },
        ];

        // Generate per-session fake trend data (X = elapsed seconds)
        for s in &demos {
            let tok_series = self
                .session_token_series
                .entry(s.session_id.clone())
                .or_default();
            let cost_series = self
                .session_cost_series
                .entry(s.session_id.clone())
                .or_default();
            for i in 0..50 {
                let elapsed = (i * 30) as f64; // 30s intervals
                let base = (s.input_tokens + s.output_tokens) as f64;
                let v = base * (i as f64 + 1.0) / 50.0; // growing over time
                tok_series.push((elapsed, v));

                let c = s
                    .cost_reported
                    .unwrap_or_else(|| self.pricing.estimate_cost(s))
                    * (i as f64 + 1.0)
                    / 50.0;
                cost_series.push((elapsed, c));
            }
        }

        // Global aggregate (X = elapsed seconds)
        for i in 0..50 {
            let elapsed = (i * 30) as f64;
            let v = 100_000.0 * (i as f64 + 1.0) / 10.0;
            self.total_token_series.push((elapsed, v));
            let c = 0.06 * (i as f64 + 1.0);
            self.total_cost_series.push((elapsed, c));
        }

        self.sessions = demos;
        self.overview_selected = 0;
    }
}
