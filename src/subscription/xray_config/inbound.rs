/// Xray inbound listener.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootInboundsItem {
    /// Bind address.
    pub listen: String,
    /// Listening port.
    pub port: i64,
    /// Inbound protocol name.
    pub protocol: String,
    /// Protocol-specific settings.
    pub settings: RootInboundsItemSettings,
    /// Optional traffic sniffing configuration.
    pub sniffing: Option<RootInboundsItemSniffing>,
    /// Unique inbound tag.
    pub tag: String,
}

/// Settings shared by supported Xray inbound protocols.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootInboundsItemSettings {
    /// Optional authentication mode.
    pub auth: Option<String>,
    /// Whether UDP is enabled.
    pub udp: Option<bool>,
    /// Xray user level.
    #[serde(rename = "userLevel")]
    pub user_level: i64,
}

/// Traffic sniffing options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootInboundsItemSniffing {
    /// Destination protocol overrides.
    #[serde(rename = "destOverride")]
    pub dest_override: Vec<String>,
    /// Enables sniffing.
    pub enabled: bool,
    /// Only use sniffed data for routing.
    #[serde(rename = "routeOnly")]
    pub route_only: bool,
}
