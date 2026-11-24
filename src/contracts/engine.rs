//! Contracts for proxy-engine adapters.
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use async_trait::async_trait;
use serde_json::Value;

use crate::prelude::error_prelude::*;

/// Fully materialized engine configuration.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Path to the engine executable.
    pub executable: PathBuf,
    /// Engine-native JSON configuration.
    pub config: Value,
    /// Maximum time allowed for startup.
    pub startup_timeout: Duration,
    /// Local SOCKS5 listener address.
    pub socks_addr: SocketAddr,
    /// Local HTTP listener address.
    pub http_addr: SocketAddr,
}

/// A running proxy-engine instance.
#[async_trait]
pub trait RunningEngine: Send {
    /// Stops the engine and waits for its process/resources to be released.
    async fn stop(&mut self) -> Result<()>;
    //noinspection ALL
    /// Returns whether the underlying engine is still alive.
    async fn is_alive(&mut self) -> Result<bool>;
}

/// Starts a proxy engine from a normalized engine configuration.
#[async_trait]
pub trait ProxyEngine: Send + Sync + 'static {
    /// Starts an engine and returns a handle that owns its lifetime.
    async fn start(&self, config: EngineConfig) -> Result<Box<dyn RunningEngine>>;
}
