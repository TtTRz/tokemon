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

**Token Monitor — AI 编码工具终端监控面板**

<br>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](http://makeapullrequest.com)

<br>

[English](README.md) · [中文](README_CN.md) · [快速开始](#-快速开始) · [功能](#-功能) · [配置](#%EF%B8%8F-配置)

</div>

<br>

## 🚀 快速开始

```bash
# 安装
cargo install --path .

# 设置 Claude Code 集成（一次性）
tokemon setup claude-code

# 重启 Claude Code，然后开始监控
tokemon
```

> [!TIP]
> 先跑 `tokemon --demo` 体验 UI，无需配置任何 Provider。

<br>

## ✨ 功能

<table>
<tr>
<td width="50%">

### 📊 实时监控
- Token 用量 — 输入 / 输出 / 缓存
- 上下文窗口 % 带色彩进度条
- 输入/输出吞吐速率 (tokens/sec)
- Pill 风格状态徽章
- 每个会话显示子代理数量

</td>
<td width="50%">

### 💰 费用追踪
- 逐 turn 精确成本累计（精确到每次 API 调用）
- 优先使用 Provider 报告的真实费用
- 内置 Claude、GPT、O3、GLM 定价
- 缓存 token 定价（写入/读取）
- 所有定价在配置文件中 — 无隐藏硬编码
- 模糊模型名匹配（兼容不同命名格式）
- 未匹配模型在状态栏高亮提示

</td>
</tr>
<tr>
<td>

### 🗂️ Overview 仪表盘
- 卡片网格布局（自动 1/2 列）
- 每个 session 带迷你趋势图
- 子代理 token 合并至父会话
- Vim 风格导航（`h/j/k/l`）
- 滚动提示 + 页码指示

</td>
<td>

### 🔍 Session 详情页
- 完整详情面板，表格对齐
- Token + 费用趋势图（共享渲染器）
- Git 分支 + 工作目录
- ANSI Shadow ASCII 艺术字头部

</td>
</tr>
<tr>
<td>

### 🌐 国际化
- 内置英文 + 简体中文
- 自动检测系统语言
- `--lang` 命令行参数或配置覆盖
- CJK 字符宽度自适应渲染

</td>
<td>

### 🔔 智能告警
- 上下文窗口 80% / 95% 阈值告警
- 费用阈值告警
- Provider 断连检测
- 进程存活检测（Claude Code）

</td>
</tr>
</table>

<br>

## 🎯 为什么需要 tokemon？

> **一个面板统御所有工具。** 不用再在终端之间切来切去，不用事后查账单。

| | 没有 tokemon | 有 tokemon |
|:--|:--|:--|
| 👀 **可见性** | 每个工具各自日志，分散各处 | 统一面板，所有 session 一目了然 |
| 💵 **费用** | 事后去账单页面查 | 逐 turn 成本追踪，cache 感知定价 |
| 📐 **上下文** | 不知道窗口还剩多少 | 实时进度条 + 80% / 95% 告警 |
| 🪟 **多会话** | 在终端之间 Alt-tab | 卡片网格 + 独立 tab |
| ⚡ **吞吐量** | 无法度量 | 输入/输出 tokens/秒 |

<br>

## ⌨️ 快捷键

| 按键 | 操作 |
|:--|:--|
| `1-9` | 跳转 tab（1=Overview，2+=各 session） |
| `Tab` / `S-Tab` | 下一个 / 上一个 tab |
| `j/k` `↑/↓` | 上下导航卡片 |
| `h/l` `←/→` | 左右导航卡片 |
| `Enter` | 打开 session 详情 tab |
| `Esc` | 返回 Overview / 退出 |
| `?` | 帮助弹窗 |
| `q` / `Ctrl+C` | 退出 |

<br>

## 🔌 已支持的工具

| Provider | 数据来源 | 安装 | 状态 |
|:--|:--|:--|:--|
| **Claude Code** | Statusline socket + JSONL 日志 | `tokemon setup claude-code` | ✅ 就绪 |
| **CodeBuddy** | Statusline socket + JSONL 日志 | `tokemon setup code-buddy` | ✅ 就绪 |
| **Codex** (OpenAI) | 日志文件监听 | — | 🔜 计划中 |
| **Custom** | 用户自定义 socket / 文件 | — | 🧩 可扩展 |

> [!NOTE]
> `tokemon setup claude-code` 会自动安装 `~/.claude/statusline.sh` 并更新 `~/.claude/settings.json`。安装后重启 Claude Code 即可。
>
> 新增 Provider：实现 `Provider` trait（约 5 个方法），注册到 `Collector`，完事。

<br>

## ⚙️ 配置

默认路径：`~/.config/tokemon/config.toml`（首次运行自动生成）

<details>
<summary><b>📄 完整配置示例</b></summary>

<br>

```toml
[general]
tick_rate_ms = 250
theme = "dark"
# locale = "zh-CN"  # 显示语言（不设则自动检测）

[providers.claude_code]
enabled = true
socket_path = "$TMPDIR/tokemon-claude.sock"
log_dirs = ["~/.claude/projects/"]

[providers.code_buddy]
enabled = true
socket_path = "$TMPDIR/tokemon-codebuddy.sock"
log_dirs = ["~/.codebuddy/projects/"]

[pricing]
default_input = 3.0    # 未知模型兜底价 ($/1M tokens)
default_output = 15.0

[pricing.models]
"claude-sonnet-4-20250514" = { input = 3.0, output = 15.0, cache_write = 3.75, cache_read = 0.30 }
"claude-opus-4-20250514"   = { input = 15.0, output = 75.0, cache_write = 18.75, cache_read = 1.50 }
"o3"                       = { input = 10.0, output = 40.0 }
"gpt-4.1"                  = { input = 2.0, output = 8.0 }

[alerts]
context_warn_pct = 80.0      # 上下文黄色警告阈值
context_crit_pct = 95.0      # 上下文红色警告阈值
cost_threshold_usd = 5.0     # 费用告警阈值
```

</details>

<br>

## 🏗️ 架构

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

## 🧱 技术栈

| | 组件 | 用途 |
|:--|:--|:--|
| 🖼️ | [ratatui](https://github.com/ratatui/ratatui) 0.29 | TUI 框架，内置 Chart/Gauge 等组件 |
| 💻 | [crossterm](https://github.com/crossterm-rs/crossterm) 0.28 | 跨平台终端后端 |
| ⚡ | [tokio](https://tokio.rs/) | 异步运行时，并发采集 Provider 数据 |
| 👁️ | [notify](https://github.com/notify-rs/notify) 7 | 文件系统监听，日志尾随 |
| 🌐 | [rust-i18n](https://github.com/longbridgeapp/rust-i18n) 3 | 编译期 i18n，YAML 翻译文件 |
| 📋 | [clap](https://github.com/clap-rs/clap) 4 | CLI 参数解析 |
| 📐 | [toml](https://github.com/toml-rs/toml) | 配置文件解析 |

<br>

## 📝 许可

[MIT](LICENSE) — 随便用。

---

<div align="center">
<sub>基于 Rust + ratatui 构建 · Catppuccin Mocha 主题</sub>
</div>
