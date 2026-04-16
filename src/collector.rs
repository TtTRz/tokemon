use tokio::sync::mpsc;

use crate::provider::{Provider, ProviderEvent};

/// Manages all registered providers, aggregates events into a single channel.
pub struct Collector {
    providers: Vec<Box<dyn Provider>>,
    tx: mpsc::Sender<ProviderEvent>,
    rx: Option<mpsc::Receiver<ProviderEvent>>,
}

impl Collector {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self {
            providers: Vec::new(),
            tx,
            rx: Some(rx),
        }
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) {
        tracing::info!("Registered provider: {}", provider.name());
        self.providers.push(provider);
    }

    /// Start all providers. Each will push events into the shared channel.
    pub async fn start_all(&self) -> anyhow::Result<()> {
        for provider in &self.providers {
            provider.start(self.tx.clone()).await?;
        }
        Ok(())
    }

    /// Take the receiver end — call only once, hand to the app event loop.
    pub fn take_event_rx(&mut self) -> Option<mpsc::Receiver<ProviderEvent>> {
        self.rx.take()
    }

    /// Stop all providers.
    pub async fn stop_all(&self) -> anyhow::Result<()> {
        for provider in &self.providers {
            provider.stop().await?;
        }
        Ok(())
    }
}
