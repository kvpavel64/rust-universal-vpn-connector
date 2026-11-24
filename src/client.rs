//! Public facade and command actor.
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use serde_json::{json, Value};
use tokio::{sync::{Mutex, RwLock}, task::JoinHandle, time::timeout};

use crate::prelude::contracts_prelude::*;
use crate::prelude::engine_prelude::*;
use crate::prelude::sing_box_prelude::*;
use crate::prelude::messaging_prelude::*;
use crate::prelude::tokio_mpsc_prelude::*;
use crate::prelude::subscription_prelude::*;
use crate::prelude::subscription_adapters_prelude::*;
use crate::prelude::types_prelude::*;
use crate::prelude::error_prelude::*;

/// Selects a local proxy endpoint for generated HTTP clients.
#[derive(Debug, Clone, Copy)]
pub enum ProxyKind {
    /// Local SOCKS5 proxy (remote DNS is enabled in `reqwest`).
    Socks5,
    /// Local HTTP CONNECT proxy.
    Http,
}

/// Builder for [`VpnClient`]. All infrastructure components can be replaced.
pub struct ClientBuilder {
    subscription: Option<String>,
    core_path: PathBuf,
    socks_addr: SocketAddr,
    http_addr: SocketAddr,
    startup_timeout: Duration,
    request_timeout: Duration,
    source: Arc<dyn SubscriptionSource>,
    parser: Arc<dyn SubscriptionParser>,
    engine: Arc<dyn ProxyEngine>,
    bus: Arc<dyn MessageBus>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self { subscription: None, core_path: PathBuf::from("sing-box.exe"),
            socks_addr: "127.0.0.1:10808".parse().expect("valid default socket address"),
            http_addr: "127.0.0.1:10809".parse().expect("valid default socket address"),
            startup_timeout: Duration::from_secs(10), request_timeout: Duration::from_secs(30),
            source: Arc::new(HttpSubscriptionSource::default()), parser: Arc::new(DefaultSubscriptionParser),
            engine: Arc::new(SingBoxEngine), bus: Arc::new(TokioMpscBus::default()) }
    }
}

impl ClientBuilder {
    /// Creates a builder with default HTTP, parser, engine, and Tokio MPSC adapters.
    pub fn new() -> Self { Self::default() }
    /// Sets the subscription URL to fetch during `build`.
    pub fn subscription(mut self, url: impl Into<String>) -> Self { self.subscription = Some(url.into()); self }
    /// Sets the path to `sing-box.exe` (or a compatible engine executable).
    pub fn core_path(mut self, path: impl Into<PathBuf>) -> Self { self.core_path = path.into(); self }
    /// Sets the local SOCKS5 listener address.
    pub fn socks_addr(mut self, addr: SocketAddr) -> Self { self.socks_addr = addr; self }
    /// Sets the local HTTP listener address.
    pub fn http_addr(mut self, addr: SocketAddr) -> Self { self.http_addr = addr; self }
    /// Sets the engine listener startup deadline.
    pub fn startup_timeout(mut self, value: Duration) -> Self { self.startup_timeout = value; self }
    /// Sets the request/reply deadline for internal control messages.
    pub fn request_timeout(mut self, value: Duration) -> Self { self.request_timeout = value; self }
    /// Replaces the subscription downloader.
    pub fn subscription_source(mut self, value: Arc<dyn SubscriptionSource>) -> Self { self.source = value; self }
    /// Replaces the subscription parser.
    pub fn subscription_parser(mut self, value: Arc<dyn SubscriptionParser>) -> Self { self.parser = value; self }
    /// Replaces the proxy engine adapter.
    pub fn proxy_engine(mut self, value: Arc<dyn ProxyEngine>) -> Self { self.engine = value; self }
    /// Replaces the broker-neutral message bus.
    pub fn message_bus(mut self, value: Arc<dyn MessageBus>) -> Self { self.bus = value; self }

    /// Fetches an optional initial subscription and starts the internal command actor.
    pub async fn build(self) -> Result<VpnClient> {
        if !self.socks_addr.ip().is_loopback() || !self.http_addr.ip().is_loopback()
            || self.socks_addr.port() == 0 || self.http_addr.port() == 0
            || self.socks_addr == self.http_addr {
            return Err(Error::InvalidProxyAddress);
        }
        let nodes = if let Some(location) = self.subscription.as_deref() {
            self.parser.parse(&self.source.fetch(location).await?)?
        } else { Vec::new() };
        let nodes = Arc::new(RwLock::new(nodes));
        let address = unique_address("vpn-client-manager")?;
        let mailbox = self.bus.register(address.clone(), MailboxCapacity::default()).await?;
        let actor_nodes = nodes.clone();
        let actor_engine = self.engine.clone();
        let actor_config = ActorConfig { core_path: self.core_path.clone(), socks_addr: self.socks_addr,
            http_addr: self.http_addr, startup_timeout: self.startup_timeout };
        let actor_address = address.clone();
        let task = tokio::spawn(async move { run_actor(mailbox, actor_address, actor_nodes, actor_engine, actor_config).await; });
        Ok(VpnClient { nodes, bus: self.bus, source: self.source, parser: self.parser, actor_address: address, socks_addr: self.socks_addr,
            http_addr: self.http_addr, request_timeout: self.request_timeout, actor_task: Mutex::new(Some(task)) })
    }
}

struct ActorConfig { core_path: PathBuf, socks_addr: SocketAddr, http_addr: SocketAddr, startup_timeout: Duration }

/// Asynchronous proxy client facade.
///
/// Control operations (`start`, `stop`, `status`) are sent as messages to an internal
/// actor. The actor depends only on [`MessageBus`], so the default in-process transport
/// can be replaced with a broker adapter. The engine process is owned by the actor.
pub struct VpnClient {
    nodes: Arc<RwLock<Vec<Node>>>,
    bus: Arc<dyn MessageBus>,
    source: Arc<dyn SubscriptionSource>,
    parser: Arc<dyn SubscriptionParser>,
    actor_address: ComponentAddress,
    socks_addr: SocketAddr,
    http_addr: SocketAddr,
    request_timeout: Duration,
    actor_task: Mutex<Option<JoinHandle<()>>>,
}

impl VpnClient {
    /// Returns a new configurable client builder.
    pub fn builder() -> ClientBuilder { ClientBuilder::new() }
    /// Returns a snapshot of the currently loaded nodes.
    pub async fn nodes(&self) -> Vec<Node> { self.nodes.read().await.clone() }
    /// Downloads and replaces the current node list.
    pub async fn add_subscription(&self, location: &str) -> Result<usize> {
        let parsed = self.parser.parse(&self.source.fetch(location).await?)?;
        let count = parsed.len();
        self.request(json!({"command":"replace_nodes","nodes":parsed})).await?;
        Ok(count)
    }
    /// Starts the selected node and waits until both local proxy listeners are ready.
    pub async fn start(&self, index: usize) -> Result<()> { self.request(json!({"command":"start","index":index})).await.map(|_| ()) }
    /// Stops the active engine process.
    pub async fn stop(&self) -> Result<()> { self.request(json!({"command":"stop"})).await.map(|_| ()) }
    /// Returns whether the engine currently reports itself alive.
    pub async fn is_running(&self) -> Result<bool> { Ok(self.request(json!({"command":"status"})).await?["running"].as_bool().unwrap_or(false)) }
    /// Returns the local proxy URI for the selected inbound type.
    pub fn local_proxy(&self, kind: ProxyKind) -> String {
        match kind { ProxyKind::Socks5 => format!("socks5h://{}", self.socks_addr), ProxyKind::Http => format!("http://{}", self.http_addr) }
    }
    /// Creates a `reqwest` client routed through the local SOCKS5 proxy.
    pub fn http_client(&self) -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder().proxy(reqwest::Proxy::all(self.local_proxy(ProxyKind::Socks5))?).build()?)
    }
    /// Creates a `reqwest` client routed through the local HTTP proxy.
    pub fn http_client_via_http_proxy(&self) -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder().proxy(reqwest::Proxy::all(self.local_proxy(ProxyKind::Http))?).build()?)
    }
    /// Gracefully stops the engine and terminates the command actor.
    pub async fn shutdown(&self) -> Result<()> {
        let result = self.request(json!({"command":"shutdown"})).await.map(|_| ());
        if let Some(task) = self.actor_task.lock().await.take() { let _ = task.await; }
        result
    }

    async fn request(&self, payload: Value) -> Result<Value> {
        let reply_address = unique_address("vpn-client-reply")?;
        let mut reply = self.bus.register(reply_address.clone(), MailboxCapacity::new(1)?) .await?;
        let mut message = Message::new(ComponentAddress::new("vpn-client-facade")?, payload);
        message.reply_to = Some(reply_address);
        let correlation_id = CorrelationId::from_message_id(message.id);
        message.correlation_id = Some(correlation_id);
        reply.send(&self.actor_address, message).await?;
        let response = timeout(self.request_timeout, reply.recv()).await.map_err(|_| Error::MessageTimeout)??.ok_or(Error::ActorStopped)?;
        if response.correlation_id != correlation_id {
            return Err(Error::MessageCorrelationMismatch { expected: correlation_id, actual: response.correlation_id });
        }
        let payload = response.payload.into_value();
        if payload["ok"].as_bool() == Some(true) { Ok(payload.get("data").cloned().unwrap_or(Value::Null)) }
        else {
            let message = payload["error"].as_str().unwrap_or("actor returned an unspecified error").to_owned();
            match payload["error_kind"].as_str().unwrap_or("engine") {
                "already_running" => Err(Error::AlreadyRunning),
                "not_running" => Err(Error::NotRunning),
                "empty_subscription" => Err(Error::EmptySubscription),
                "node_index" => Err(Error::NodeIndex(payload["node_index"].as_u64().unwrap_or(usize::MAX as u64) as usize)),
                "core_exited" => Err(Error::CoreExited),
                "startup_timeout" => Err(Error::StartupTimeout),
                "invalid_core_path" => Err(Error::InvalidCorePath),
                "invalid_proxy_address" => Err(Error::InvalidProxyAddress),
                "actor_protocol" => Err(Error::RemoteActorFailure(message)),
                "engine" => Err(Error::RemoteEngineFailure(message)),
                _ => Err(Error::RemoteActorFailure(message)),
            }
        }
    }
}

impl Drop for VpnClient {
    fn drop(&mut self) {
        if let Some(task) = self.actor_task.get_mut().take() { task.abort(); }
    }
}

async fn run_actor(mut mailbox: Box<dyn Mailbox>, address: ComponentAddress,
    nodes: Arc<RwLock<Vec<Node>>>, engine: Arc<dyn ProxyEngine>, config: ActorConfig) {
    let mut running: Option<Box<dyn crate::contracts::RunningEngine>> = None;
    while let Ok(Some(message)) = mailbox.recv().await {
        let payload = message.payload.as_value();
        let command = payload["command"].as_str().unwrap_or("");
        let mut shutdown = false;
        let result: Result<Value> = match command {
            "start" => {
                if running.is_some() { Err(Error::AlreadyRunning) } else {
                    let index = payload["index"].as_u64().unwrap_or(u64::MAX) as usize;
                    match selected_node_config(&nodes, index, &config).await {
                        Ok(engine_config) => match engine.start(engine_config).await { Ok(handle) => { running = Some(handle); Ok(Value::Null) }, Err(error) => Err(error) },
                        Err(error) => Err(error),
                    }
                }
            }
            "replace_nodes" => match serde_json::from_value::<Vec<Node>>(payload["nodes"].clone()) {
                Ok(replacement) if !replacement.is_empty() => { *nodes.write().await = replacement; Ok(Value::Null) }
                Ok(_) => Err(Error::EmptySubscription),
                Err(error) => Err(Error::Json(error)),
            },
            "stop" => match running.as_mut() { Some(handle) => { let result = handle.stop().await; running = None; result.map(|_| Value::Null) }, None => Err(Error::NotRunning) },
            "status" => {
                let status = match running.as_mut() { Some(handle) => Some(handle.is_alive().await), None => None };
                match status {
                    Some(Ok(true)) => Ok(json!({"running":true})),
                    Some(Ok(false)) => { running = None; Ok(json!({"running":false})) },
                    Some(Err(error)) => Err(error),
                    None => Ok(json!({"running":false})),
                }
            },
            "shutdown" => { shutdown = true; if let Some(mut handle) = running.take() { handle.stop().await.map(|_| Value::Null) } else { Ok(Value::Null) } },
            _ => Err(Error::UnknownActorCommand(command.to_owned())),
        };
        if let Some(reply_to) = message.reply_to.clone() {
            let response_payload = match result { Ok(data) => json!({"ok":true,"data":data}), Err(error) => actor_error_payload(error) };
            let mut response = Message::new(address.clone(), response_payload);
            response.correlation_id = message.correlation_id;
            let _ = mailbox.send(&reply_to, response).await;
        }
        if shutdown { break; }
    }
}


fn actor_error_payload(error: Error) -> Value {
    let (kind, node_index) = match &error {
        Error::AlreadyRunning => ("already_running", None),
        Error::NotRunning => ("not_running", None),
        Error::EmptySubscription => ("empty_subscription", None),
        Error::NodeIndex(index) => ("node_index", Some(*index)),
        Error::CoreExited => ("core_exited", None),
        Error::StartupTimeout => ("startup_timeout", None),
        Error::InvalidCorePath => ("invalid_core_path", None),
        Error::InvalidProxyAddress => ("invalid_proxy_address", None),
        Error::UnknownActorCommand(_) | Error::Json(_) => ("actor_protocol", None),
        _ => ("engine", None),
    };
    json!({"ok": false, "error_kind": kind, "error": error.to_string(), "node_index": node_index})
}

async fn selected_node_config(nodes: &RwLock<Vec<Node>>, index: usize, config: &ActorConfig) -> Result<EngineConfig> {
    let node = nodes.read().await.get(index).cloned().ok_or(Error::NodeIndex(index))?;
    make_engine_config(node, config)
}

fn make_engine_config(node: Node, config: &ActorConfig) -> Result<EngineConfig> {
    let mut outbound = node.outbound;
    outbound["tag"] = json!("proxy");
    let value = json!({
        "log":{"level":"warn","timestamp":true},
        "inbounds":[
            {"type":"socks","tag":"socks-in","listen":config.socks_addr.ip().to_string(),"listen_port":config.socks_addr.port(),"udp":true},
            {"type":"http","tag":"http-in","listen":config.http_addr.ip().to_string(),"listen_port":config.http_addr.port()}
        ],
        "outbounds":[outbound,{"type":"direct","tag":"direct"}],
        "route":{"final":"proxy"}
    });
    Ok(EngineConfig { executable: config.core_path.clone(), config: value, startup_timeout: config.startup_timeout,
        socks_addr: config.socks_addr, http_addr: config.http_addr })
}

fn unique_address(prefix: &str) -> Result<ComponentAddress> {
    ComponentAddress::new(format!("{prefix}-{}", crate::MessageId::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_config_has_both_local_inbounds_and_proxy_route() {
        let node = Node { name: "test".into(), protocol: crate::Protocol::Vless,
            outbound: json!({"type":"vless","server":"example.org","server_port":443,"uuid":"uuid"}) };
        let cfg = ActorConfig { core_path: "sing-box.exe".into(), socks_addr: "127.0.0.1:10808".parse().unwrap(),
            http_addr: "127.0.0.1:10809".parse().unwrap(), startup_timeout: Duration::from_secs(5) };
        let engine = make_engine_config(node, &cfg).unwrap();
        assert_eq!(engine.config["inbounds"].as_array().unwrap().len(), 2);
        assert_eq!(engine.config["route"]["final"], "proxy");
    }
}
