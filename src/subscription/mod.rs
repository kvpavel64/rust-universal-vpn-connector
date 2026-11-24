//! Subscription domain: fetching, parsing, and engine-specific configuration schemas.
mod http_source;
mod parser;
pub mod xray_config;

pub use http_source::HttpSubscriptionSource;
pub use parser::DefaultSubscriptionParser;
pub use xray_config::{Nodes as XrayNodes, Root as XrayConfig};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Protocol understood by the default sing-box adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// VLESS proxy protocol.
    Vless,
    /// VMess proxy protocol.
    Vmess,
    /// Trojan proxy protocol.
    Trojan,
    /// Shadowsocks proxy protocol.
    Shadowsocks,
    /// SOCKS proxy protocol.
    Socks5,
    /// HTTP proxy protocol.
    Http,
    /// Hysteria2 proxy protocol.
    Hysteria2,
}

impl Protocol {
    /// Returns the sing-box outbound type string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vless => "vless", Self::Vmess => "vmess", Self::Trojan => "trojan",
            Self::Shadowsocks => "shadowsocks", Self::Socks5 => "socks", Self::Http => "http",
            Self::Hysteria2 => "hysteria2",
        }
    }
}

/// A normalized proxy node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Human-readable node label.
    pub name: String,
    /// Normalized protocol identifier.
    pub protocol: Protocol,
    /// sing-box-compatible outbound object.
    pub outbound: Value,
}
