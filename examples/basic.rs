//! Minimal end-to-end usage. Run with `cargo run --example basic` after configuring
//! a real subscription URL and placing `sing-box.exe` beside the executable.
use std::{net::SocketAddr, time::Duration};
use vpn_client::VpnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socks: SocketAddr = "127.0.0.1:10808".parse()?;
    let http: SocketAddr = "127.0.0.1:10809".parse()?;
    let client = VpnClient::builder()
        .subscription("https://example.org/my-subscription")
        .core_path("./sing-box.exe")
        .socks_addr(socks)
        .http_addr(http)
        .startup_timeout(Duration::from_secs(15))
        .build()
        .await?;

    for (index, node) in client.nodes().await.iter().enumerate() {
        println!("{index}: {} ({:?})", node.name, node.protocol);
    }

    client.start(0).await?;
    let http_client = client.http_client()?;
    let response = http_client.get("https://api.ipify.org").send().await?;
    println!("Exit IP: {}", response.text().await?);
    client.shutdown().await?;
    Ok(())
}
