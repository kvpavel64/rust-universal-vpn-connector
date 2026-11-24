//! Shows how to replace subscription acquisition and parsing.
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;
use vpn_client::{Node, Protocol, Result, SubscriptionParser, SubscriptionSource, VpnClient};

struct EmbeddedSource;
#[async_trait]
impl SubscriptionSource for EmbeddedSource {
    async fn fetch(&self, _: &str) -> Result<String> { Ok("embedded".to_owned()) }
}

struct EmbeddedParser;
impl SubscriptionParser for EmbeddedParser {
    fn parse(&self, _: &str) -> Result<Vec<Node>> {
        Ok(vec![Node {
            name: "embedded-node".to_owned(),
            protocol: Protocol::Vless,
            outbound: json!({"type":"vless","server":"proxy.example.org","server_port":443,
                "uuid":"00000000-0000-0000-0000-000000000000","tls":{"enabled":true}}),
        }])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VpnClient::builder()
        .subscription("embedded://profile")
        .subscription_source(Arc::new(EmbeddedSource))
        .subscription_parser(Arc::new(EmbeddedParser))
        .core_path("./sing-box.exe")
        .build().await?;
    println!("Loaded {} node(s)", client.nodes().await.len());
    client.shutdown().await?;
    Ok(())
}
