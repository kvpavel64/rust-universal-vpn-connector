//! A composable asynchronous proxy client for Windows and other platforms supported by
//! [sing-box](https://github.com/SagerNet/sing-box).
//!
//! # Design
//!
//! The crate is split into replaceable domain components. Public traits live in
//! [`contracts`]; default implementations live in [`subscription`], [`engine`], and
//! [`messaging`]. The client facade sends commands and subscription updates to its
//! manager actor through [`MessageBus`]. The manager currently invokes the selected
//! [`ProxyEngine`] adapter through its trait contract; splitting that adapter into a
//! separately hosted actor is a compatible next step. The default bus uses Tokio MPSC;
//! applications can provide a broker-backed implementation without changing the facade.
//!
//! # Quick start
//!
//! ```no_run
//! use std::{net::SocketAddr, time::Duration};
//! use vpn_client::VpnClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = VpnClient::builder()
//!     .subscription("https://example.org/subscription")
//!     .core_path("./sing-box.exe")
//!     .socks_addr("127.0.0.1:10808".parse::<SocketAddr>()?)
//!     .startup_timeout(Duration::from_secs(15))
//!     .build().await?;
//! client.start(0).await?;
//! let http = client.http_client()?;
//! let response = http.get("https://api.ipify.org").send().await?;
//! println!("{}", response.text().await?);
//! client.stop().await?;
//! # Ok(())
//! # }
//! ```
//!
//! The library manages the local proxy process. It does not install a TUN adapter or
//! modify system routing. Applications must explicitly use the returned HTTP client or
//! connect to the local SOCKS5/HTTP listener.

#![deny(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod client;
pub mod contracts;
pub mod engine;
mod error;
pub mod messaging;
pub mod prelude;
pub mod subscription;
pub mod types;

pub use client::{ClientBuilder, ProxyKind, VpnClient};
pub use contracts::{
    EngineConfig, Mailbox, Message, MessageBus, MessagePayload, ProxyEngine, RunningEngine,
    SubscriptionParser, SubscriptionSource,
};
pub use engine::SingBoxEngine;
pub use error::{Error, Result};
pub use messaging::TokioMpscBus;
pub use subscription::{DefaultSubscriptionParser, HttpSubscriptionSource, XrayConfig, XrayNodes};
pub use subscription::{Node, Protocol};
pub use types::{ComponentAddress, CorrelationId, MailboxCapacity, MessageId};
