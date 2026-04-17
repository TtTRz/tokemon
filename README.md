<div align="center">

<br>

<pre>
████████╗ ██████╗ ██╗  ██╗███████╗███╗   ███╗ ██████╗ ███╗   ██╗
╚══██╔══╝██╔═══██╗██║ ██╔╝██╔════╝████╗ ████║██╔═══██╗████╗  ██║
   ██║   ██║   ██║█████╔╝ █████╗  ██╔████╔██║██║   ██║██╔██╗ ██║
   ██║   ██║   ██║██╔═██╗ ██╔══╝  ██║╚██╔╝██║██║   ██║██║╚██╗██║
   ██║   ╚██████╔╝██║  ██╗███████╗██║ ╚═╝ ██║╚██████╔╝██║ ╚████║
   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
</pre>

**Token Monitor — Terminal dashboard for AI coding tools**

<br>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](http://makeapullrequest.com)

<br>

[English](README.md) · [中文](README_CN.md) · [Quick Start](#-quick-start) · [Features](#-features) · [Configuration](#%EF%B8%8F-configuration)

</div>

<br>

## 🚀 Quick Start

```bash
# Install from crates.io
cargo install tokemon-cli

# Or build from source
git clone https://github.com/TtTRz/tokemon.git
cd tokemon
cargo install --path .
```

```bash
# Set up Claude Code integration (one-time)
tokemon setup claude-code

# Restart Claude Code, then start monitoring
tokemon
```

> [!TIP]
> Run `tokemon --demo` first to explore the UI without any provider setup.

<br>

## ✨ Features

<table>
<tr>
<td width="50%">

### 📊 Real-time Monitoring
- Token usage — input / output / cached
- Context window % with color-coded gauge
- Input/output throughput (tokens/sec)
- Session status with pill badge
- Subagent count per session

</td>
<td width="50%">

### 💰 Cost Tracking
- Per-turn cost accumulation (accurate to API call level)
- Reported cost from provider when available
- Built-in pricing for Claude, GPT, O3, GLM
- Cache token pricing (write / read)
- All pricing in config file — no hidden hardcoding
- Fuzzy model name matching (handles naming variants)
- Unmatched models highlighted in status bar

</td>
</tr>
<tr>
<td>

### 🗂️ Overview Dashboard
- Card grid layout (auto 1/2 columns)
- Per-session mini trend charts
- Subagent tokens merged into parent session
- Vim-style navigation (`h/j/k/l`)
- Scroll hints with page indicator

</td>
<td>

### 🔍 Session Detail Tabs
- Full detail panel, table-aligned fields
- Token + cost trend charts (shared renderer)
- Git branch + working directory
- ANSI Shadow ASCII art header

</td>
</tr>
<tr>
<td>

### 🌐 Internationalization
- English + 简体中文 built-in
- Auto-detect system locale
- `--lang` CLI flag or config override
- CJK-aware column width rendering

</td>
<td>

### 🔔 Smart Alerts
- Context window 80% / 95% thresholds
- Cost threshold alerts
- Provider disconnect detection
- Process-alive session status (Claude Code)

</td>
</tr>
</table>

<br>

## 🎯 Why tokemon?

> **One dashboard to rule them all.** Stop alt-tabbing between terminals. Stop checking billing portals after the fact.

| | Without tokemon | With tokemon |
|:--|:--|:--|
| 👀 **Visibility** | Scattered logs per tool | Unified dashboard, all sessions at a glance |
| 💵 **Cost** | Check billing portal later | Per-turn cost tracking with cache-aware pricing |
| 📐 **Context** | No idea how full the window is | Live gauge with 80% / 95% alerts |
| 🪟 **Multi-session** | Alt-tab between terminals | Card grid + per-session tabs |
| ⚡ **Speed** | No way to measure throughput | Input/output tokens per second |

<br>

## ⌨️ Keybindings

| Key | Action |
|:--|:--|
| `1-9` | Jump to tab (1=Overview, 2+=sessions) |
| `Tab` / `S-Tab` | Next / previous tab |
| `j/k` `↑/↓` | Navigate cards up/down |
| `h/l` `←/→` | Navigate cards left/right |
| `Enter` | Open session detail tab |
| `Esc` | Back to Overview / Quit |
| `?` | Help overlay |
| `q` / `Ctrl+C` | Quit |

<br>

## 🔌 Supported Providers

| Provider | Data Source | Setup | Status |
|:--|:--|:--|:--|
| **Claude Code** | Statusline socket + JSONL logs | `tokemon setup claude-code` | ✅ Ready |
| **CodeBuddy** | Statusline socket + JSONL logs | `tokemon setup code-buddy` | ✅ Ready |
| **Codex** (OpenAI) | Log file watching | — | 🔜 Planned |
| **Custom** | User-defined socket / file | — | 🧩 Extensible |

> [!NOTE]
> `tokemon setup claude-code` automatically installs `~/.claude/statusline.sh` and updates `~/.claude/settings.json`. Restart Claude Code after setup.
>
> Adding a new provider: implement the `Provider` trait (~5 methods), register in `Collector`, done.

<br>

## ⚙️ Configuration

Default path: `~/.config/tokemon/config.toml` (auto-generated on first run)

<details>
<summary><b>📄 Full config example</b></summary>

<br>

```toml
[general]
tick_rate_ms = 250
theme = "dark"
# locale = "zh-CN"  # Display language (auto-detected if omitted)

[providers.claude_code]
enabled = true
socket_path = "$TMPDIR/tokemon-claude.sock"
log_dirs = ["~/.claude/projects/"]

[providers.code_buddy]
enabled = true
socket_path = "$TMPDIR/tokemon-codebuddy.sock"
log_dirs = ["~/.codebuddy/projects/"]

[pricing]
default_input = 3.0    # $/1M tokens fallback
default_output = 15.0

[pricing.models]
"claude-sonnet-4-20250514" = { input = 3.0, output = 15.0, cache_write = 3.75, cache_read = 0.30 }
"claude-opus-4-20250514"   = { input = 15.0, output = 75.0, cache_write = 18.75, cache_read = 1.50 }
"o3"                       = { input = 10.0, output = 40.0 }
"gpt-4.1"                  = { input = 2.0, output = 8.0 }

[alerts]
context_warn_pct = 80.0
context_crit_pct = 95.0
cost_threshold_usd = 5.0
```

</details>

<br>

## 🏗️ Architecture

```mermaid
graph TD
    A[App - ratatui TUI] --> B[Collector]
    B --> C[Claude Code Provider]
    B --> D[CodeBuddy Provider]
    A --> F[Pricing Engine]
    A --> G[Alert Engine]
    A --> H[i18n - rust-i18n]
    C -->|ProviderEvent| B
    D -->|ProviderEvent| B
    style A fill:#89b4fa,color:#1e1e2e
    style B fill:#a6e3a1,color:#1e1e2e
    style F fill:#fab387,color:#1e1e2e
    style G fill:#f38ba8,color:#1e1e2e
    style H fill:#cba6f7,color:#1e1e2e
```

<br>

## 🧱 Tech Stack

| | Component | Why |
|:--|:--|:--|
| 🖼️ | [ratatui](https://github.com/ratatui/ratatui) 0.29 | TUI framework with Chart, Gauge, built-in widgets |
| 💻 | [crossterm](https://github.com/crossterm-rs/crossterm) 0.28 | Cross-platform terminal backend |
| ⚡ | [tokio](https://tokio.rs/) | Async runtime for concurrent provider collection |
| 👁️ | [notify](https://github.com/notify-rs/notify) 7 | File system watching for log tailing |
| 🌐 | [rust-i18n](https://github.com/longbridgeapp/rust-i18n) 3 | Compile-time i18n with YAML translation files |
| 📋 | [clap](https://github.com/clap-rs/clap) 4 | CLI argument parsing |
| 📐 | [toml](https://github.com/toml-rs/toml) | Config file parsing |

<br>

## 📝 License

[MIT](LICENSE) — do whatever you want.

---

<div align="center">
<sub>Built with Rust + ratatui · Catppuccin Mocha theme</sub>
</div>
