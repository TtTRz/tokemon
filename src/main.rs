mod alert;
mod app;
mod collector;
mod config;
mod model;
mod pricing;
mod provider;
mod setup;
mod ui;

use std::time::Duration;

use clap::{Parser, Subcommand};
use crossterm::event::EventStream;
use futures_lite::StreamExt;

use crate::alert::AlertEngine;
use crate::app::App;
use crate::collector::Collector;
use crate::config::Config;
use crate::pricing::PricingEngine;
use crate::provider::claude_code::ClaudeCodeProvider;

#[derive(Parser, Debug)]
#[command(
    name = "tokemon",
    version,
    about = "Token Monitor — Terminal dashboard for AI coding tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Demo mode: show fake sessions for UI development
    #[arg(long, default_value_t = false)]
    demo: bool,

    /// Tick rate in milliseconds
    #[arg(long)]
    tick_rate: Option<u64>,

    /// Config file path (default: ~/.config/tokemon/config.toml)
    #[arg(long, short)]
    config: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Set up statusline bridge for a provider
    Setup {
        /// Provider to configure (e.g. "claude-code")
        provider: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle subcommands first (no TUI needed)
    if let Some(Commands::Setup { provider }) = &cli.command {
        return setup::run(provider);
    }

    // Initialize tracing to stderr (TUI owns stdout)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tokemon=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    let config = Config::load();
    let tick_rate = Duration::from_millis(cli.tick_rate.unwrap_or(config.general.tick_rate_ms));

    let pricing = PricingEngine::new(&config.pricing);
    let alert_engine = AlertEngine::new(&config.alerts);
    let mut app = App::new(pricing, alert_engine);

    // Set up collector
    let mut collector = Collector::new(256);
    if config.providers.claude_code.enabled {
        let cc = ClaudeCodeProvider::new(
            config.providers.claude_code.socket_path.clone(),
            config.providers.claude_code.log_dirs.clone(),
        );
        collector.register(Box::new(cc));
    }

    let mut collector_rx = collector.take_event_rx().expect("event rx already taken");
    collector.start_all().await?;

    if cli.demo {
        app.add_demo_sessions();
    }

    // Initialize terminal
    let mut terminal = ratatui::init();
    let mut event_stream = EventStream::new();
    let mut tick = tokio::time::interval(tick_rate);

    // Main event loop
    loop {
        tokio::select! {
            Some(Ok(term_event)) = event_stream.next() => {
                app.handle_terminal_event(term_event);
            }
            Some(provider_event) = collector_rx.recv() => {
                app.handle_provider_event(provider_event);
            }
            _ = tick.tick() => {
                app.check_alerts();
                terminal.draw(|frame| app.render(frame))?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    collector.stop_all().await?;
    ratatui::restore();
    Ok(())
}
