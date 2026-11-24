//! Contract-level tests using fake adapters. No network or external engine is required.
use std::{net::SocketAddr, path::PathBuf, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}, time::Duration};
use async_trait::async_trait;
use serde_json::json;
use vpn_client::{EngineConfig, Error, MessageBus, Node, Protocol, ProxyEngine, Result, RunningEngine,
    SubscriptionParser, SubscriptionSource, TokioMpscBus, VpnClient};

struct MemorySource;
#[async_trait]
impl SubscriptionSource for MemorySource {
    async fn fetch(&self, _: &str) -> Result<String> { Ok("memory-document".into()) }
}
struct MemoryParser;
impl SubscriptionParser for MemoryParser {
    fn parse(&self, _: &str) -> Result<Vec<Node>> {
        Ok(vec![Node { name: "fixture".into(), protocol: Protocol::Vless,
            outbound: json!({"type":"vless","server":"127.0.0.1","server_port":443,"uuid":"fixture"}) }])
    }
}
#[derive(Default)]
struct FakeEngine { starts: AtomicUsize }
struct FakeRunning { alive: AtomicBool }
#[async_trait]
impl RunningEngine for FakeRunning {
    async fn stop(&mut self) -> Result<()> { self.alive.store(false, Ordering::SeqCst); Ok(()) }
    async fn is_alive(&mut self) -> Result<bool> { Ok(self.alive.load(Ordering::SeqCst)) }
}
#[async_trait]
impl ProxyEngine for FakeEngine {
    async fn start(&self, config: EngineConfig) -> Result<Box<dyn RunningEngine>> {
        assert_eq!(config.config["inbounds"].as_array().unwrap().len(), 2);
        self.starts.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(FakeRunning { alive: AtomicBool::new(true) }))
    }
}

#[tokio::test]
async fn facade_uses_injected_components_and_actor_messages() {
    let engine = Arc::new(FakeEngine::default());
    let bus: Arc<dyn MessageBus> = Arc::new(TokioMpscBus::default());
    let client = VpnClient::builder()
        .subscription("memory://fixture")
        .subscription_source(Arc::new(MemorySource))
        .subscription_parser(Arc::new(MemoryParser))
        .proxy_engine(engine.clone())
        .message_bus(bus)
        .request_timeout(Duration::from_secs(2))
        .core_path(PathBuf::from("fake-engine.exe"))
        .build().await.unwrap();

    assert_eq!(client.nodes().await.len(), 1);
    assert_eq!(client.add_subscription("memory://refresh").await.unwrap(), 1);
    assert!(!client.is_running().await.unwrap());
    client.start(0).await.unwrap();
    assert!(client.is_running().await.unwrap());
    assert_eq!(engine.starts.load(Ordering::SeqCst), 1);
    assert!(matches!(client.start(0).await, Err(Error::AlreadyRunning)));
    client.stop().await.unwrap();
    assert!(!client.is_running().await.unwrap());
    client.shutdown().await.unwrap();
}

#[tokio::test]
async fn actor_reports_invalid_node_index() {
    let client = VpnClient::builder()
        .subscription_source(Arc::new(MemorySource))
        .subscription_parser(Arc::new(MemoryParser))
        .proxy_engine(Arc::new(FakeEngine::default()))
        .build().await.unwrap();
    assert!(matches!(client.start(42).await, Err(Error::NodeIndex(42))));
    client.shutdown().await.unwrap();
}

#[tokio::test]
async fn rejects_non_loopback_listener_addresses() {
    let result = VpnClient::builder()
        .socks_addr("0.0.0.0:10808".parse().unwrap())
        .build().await;
    assert!(matches!(result, Err(Error::InvalidProxyAddress)));
}

#[test]
fn socket_address_type_is_platform_independent() {
    let _: SocketAddr = "127.0.0.1:10808".parse().unwrap();
}
