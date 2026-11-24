//! Default adapter that manages a sing-box child process.
use std::{process::Stdio, time::Duration};
use async_trait::async_trait;
use tokio::{net::TcpStream, process::{Child, Command}, time::{sleep, timeout}};
use tempfile::TempDir;

use crate::prelude::engine_prelude::*;
use crate::prelude::error_prelude::*;

/// Proxy engine adapter for the `sing-box` executable.
///
/// The executable is supplied by the application and is not downloaded automatically.
#[derive(Debug, Clone)]
pub struct SingBoxEngine;

#[async_trait]
impl ProxyEngine for SingBoxEngine {
    async fn start(&self, config: EngineConfig) -> Result<Box<dyn RunningEngine>> {
        if !config.executable.is_file() { return Err(Error::InvalidCorePath); }
        let temp = tempfile::Builder::new().prefix("vpn-client-").tempdir()?;
        let config_path = temp.path().join("config.json");
        tokio::fs::write(&config_path, serde_json::to_vec_pretty(&config.config)?).await?;
        let mut child = Command::new(&config.executable)
            .arg("run").arg("-c").arg(&config_path)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .kill_on_drop(true).spawn()?;
        let readiness = async {
            loop {
                if child.try_wait()?.is_some() { return Err(Error::CoreExited); }
                let socks_ready = TcpStream::connect(config.socks_addr).await.is_ok();
                let http_ready = TcpStream::connect(config.http_addr).await.is_ok();
                if socks_ready && http_ready { return Ok(()); }
                sleep(Duration::from_millis(100)).await;
            }
        };
        match timeout(config.startup_timeout, readiness).await {
            Ok(Ok(())) => Ok(Box::new(SingBoxProcess { child, _temp: temp })),
            Ok(Err(error)) => { let _ = child.kill().await; let _ = child.wait().await; Err(error) }
            Err(_) => { let _ = child.kill().await; let _ = child.wait().await; Err(Error::StartupTimeout) }
        }
    }
}

struct SingBoxProcess { child: Child, _temp: TempDir }

#[async_trait]
impl RunningEngine for SingBoxProcess {
    async fn stop(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_none() { self.child.kill().await?; }
        let _ = self.child.wait().await?;
        Ok(())
    }
    async fn is_alive(&mut self) -> Result<bool> { Ok(self.child.try_wait()?.is_none()) }
}

impl Drop for SingBoxProcess {
    fn drop(&mut self) { let _ = self.child.start_kill(); }
}
