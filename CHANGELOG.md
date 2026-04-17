# Changelog

## [0.2.2] - 2026-04-17

### Added
- Detail panel title now shows provider name, session id and status icon (aligned with overview cards)
- Bottom status bar shows current model pricing (input/output/cache rates per 1M tokens)
- Warning in status bar when model has no pricing configured
- Subagent display split into active/total count (e.g. `⑂ 2 active / 3 total`)
- Active subagent detection via file mtime (< 5 min = active)

### Changed
- Model pricing match upgraded from prefix match to fuzzy token matching
  - Handles naming variants like `claude-4.6-opus` vs `claude-opus-4-6`
  - Strips `[...]` suffix (context window marker) before matching
- `get_price()` returns `Option` — unmatched models no longer silently fall back to default rates
- Removed alert system from bottom bar, replaced with model pricing display

### Fixed
- Cleaned up verbose/informal code comments

## [0.2.1] - 2026-04-17

### Added
- **i18n support** (rust-i18n): English + Simplified Chinese
  - `--lang` CLI flag, config `locale` field, sys-locale auto-detection
  - CJK-aware column width rendering via unicode-width
- **Subagent merging**: subagent tokens accumulated into parent session (Claude Code + CodeBuddy)
- Subagent count displayed per session in overview cards and detail panel
- Session status **process-alive detection** for Claude Code (checks `/tmp/claude-{uid}/` directory)
- Per-turn cost accumulation in JSONL parsing (accurate to individual API call)
- Auto-generate `~/.config/tokemon/config.toml` on first run with all default pricing
- Dead session cleanup: series data freed after 30 min of Done/Disconnected status
- Dir row in overview cards (below Cost row)

### Changed
- Session status thresholds: Active < 5 min, Idle 5–30 min, Done > 30 min (was 1 min / 10 min)
- All model pricing moved from hardcoded builtins to config file
- Shared `pad_r()` with unicode-width (was duplicated in 3 files, byte-based)
- Shared `theme.rs` for Catppuccin Mocha colors (was duplicated in 7 files)
- Shared `ui/shared.rs` for common UI functions (fmt_tok, ctx_color, render_status_badge, etc.)
- Unified chart renderer: overview cards and detail tab share `trend_chart.rs`
- History tab sorted by timestamp descending
- Chart Y-axis label width aligned between token and cost charts
- Monotonic clamp for cumulative chart series (prevents value regression)

### Fixed
- Cost estimation jumping: clamp cached tokens to input_tokens before subtraction
- Welcome screen showing header/tabs when only done sessions exist
- Scroll hint width calculation for CJK characters

## [0.1.0] - 2026-04-16

### Added
- Initial release
- Real-time token monitoring: input / output / cached / context window %
- Cost estimation with built-in pricing for Claude, GPT, O3, GLM
- Overview dashboard with card grid layout (auto 1/2 columns)
- Per-session mini trend charts (tokens + cost)
- Session detail tabs with full stats and trend charts
- Vim-style navigation (h/j/k/l)
- Help overlay, scroll hints, pill-style status badges
- ANSI Shadow ASCII art header with right-aligned aggregate stats
- Providers: Claude Code (statusline socket + JSONL logs)
- Configurable via `~/.config/tokemon/config.toml`
- Alerts: context window threshold, cost threshold
- Demo mode (`--demo`)

## [0.0.2] - 2026-04-16

### Added
- CodeBuddy provider (statusline socket + JSONL logs)
- `tokemon setup code-buddy` command
- GLM model pricing

### Fixed
- Socket JSON format handling
- Token accounting and session merge logic

## [0.0.1] - 2026-04-16

- Initial commit: Token Monitor TUI dashboard
