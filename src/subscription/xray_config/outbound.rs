/// Xray outbound proxy definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItem {
    /// Unique outbound tag.
    pub tag: String,
    /// Outbound protocol name.
    pub protocol: String,
    /// Optional protocol settings.
    pub settings: Option<RootOutboundsItemSettings>,
    /// Optional transport and TLS settings.
    #[serde(rename = "streamSettings")]
    pub stream_settings: Option<RootOutboundsItemStreamSettings>,
    /// Optional multiplexing settings.
    pub mux: Option<RootOutboundsItemMux>,
}

/// Protocol-specific outbound settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemSettings {
    /// VLESS/VMess destination definitions.
    pub vnext: Option<Vec<RootOutboundsItemSettingsVnextItem>>,
    /// Domain resolution strategy.
    #[serde(rename = "domainStrategy")]
    pub domain_strategy: Option<String>,
    /// Protocol response settings.
    pub response: Option<RootOutboundsItemSettingsResponse>,
    /// Destination address for simple proxy protocols.
    pub address: Option<String>,
    /// Destination port.
    pub port: Option<i64>,
    /// Protocol version.
    pub version: Option<i64>,
    /// Shadowsocks server definitions.
    pub servers: Option<Vec<RootOutboundsItemSettingsServersItem>>,
}

/// VLESS/VMess destination server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemSettingsVnextItem {
    /// Server hostname or IP.
    pub address: String,
    /// Server port.
    pub port: i64,
    /// User credentials.
    pub users: Vec<RootOutboundsItemSettingsVnextItemUsersItem>,
}

/// VLESS/VMess user credentials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemSettingsVnextItemUsersItem {
    /// User UUID.
    pub id: String,
    /// Encryption mode.
    pub encryption: String,
    /// Optional flow name (empty string when absent in source exports).
    #[serde(default)]
    pub flow: String,
}

/// Protocol response configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemSettingsResponse {
    /// Response type.
    pub r#type: String,
}

/// Shadowsocks server configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemSettingsServersItem {
    /// Server hostname or IP.
    pub address: String,
    /// Server port.
    pub port: i64,
    /// Server password.
    pub password: String,
    /// Cipher method.
    pub method: String,
    /// Enable UDP-over-TCP.
    pub uot: bool,
    /// UDP-over-TCP protocol version.
    #[serde(rename = "UoTVersion")]
    pub uo_tversion: i64,
}

/// Outbound transport and security settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettings {
    /// Transport network (`tcp`, `ws`, `xhttp`, etc.).
    pub network: String,
    /// Optional TCP transport settings.
    #[serde(rename = "tcpSettings")]
    pub tcp_settings: Option<RootOutboundsItemStreamSettingsTcpSettings>,
    /// Security mode.
    pub security: String,
    /// Optional REALITY settings.
    #[serde(rename = "realitySettings")]
    pub reality_settings: Option<RootOutboundsItemStreamSettingsRealitySettings>,
    /// Optional Hysteria transport settings.
    #[serde(rename = "hysteriaSettings")]
    pub hysteria_settings: Option<RootOutboundsItemStreamSettingsHysteriaSettings>,
    /// Optional TLS settings.
    #[serde(rename = "tlsSettings")]
    pub tls_settings: Option<RootOutboundsItemStreamSettingsTlsSettings>,
    /// Optional final-mask settings.
    pub finalmask: Option<RootOutboundsItemStreamSettingsFinalmask>,
    /// Optional WebSocket settings.
    #[serde(rename = "wsSettings")]
    pub ws_settings: Option<RootOutboundsItemStreamSettingsWsSettings>,
    /// Optional XHTTP settings.
    #[serde(rename = "xhttpSettings")]
    pub xhttp_settings: Option<RootOutboundsItemStreamSettingsXhttpSettings>,
}

/// Empty TCP settings object, retained for compatibility with Xray JSON.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsTcpSettings {}

/// REALITY security settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsRealitySettings {
    /// TLS server name.
    #[serde(rename = "serverName")]
    pub server_name: String,
    /// REALITY public key.
    #[serde(rename = "publicKey")]
    pub public_key: String,
    /// REALITY short ID.
    #[serde(rename = "shortId")]
    pub short_id: String,
    /// Client TLS fingerprint.
    pub fingerprint: String,
}

/// Hysteria transport settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsHysteriaSettings {
    /// Hysteria protocol version.
    pub version: i64,
    /// Authentication token.
    pub auth: String,
}

/// TLS settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsTlsSettings {
    /// TLS server name.
    #[serde(rename = "serverName")]
    pub server_name: String,
    /// Client fingerprint.
    pub fingerprint: String,
    /// ALPN protocols.
    pub alpn: Vec<String>,
    /// Optional peer certificate name constraint.
    #[serde(rename = "verifyPeerCertByName")]
    pub verify_peer_cert_by_name: Option<String>,
}

/// Final-mask transport settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsFinalmask {
    /// Optional QUIC parameters.
    #[serde(rename = "quicParams")]
    pub quic_params: Option<RootOutboundsItemStreamSettingsFinalmaskQuicParams>,
    /// Optional TCP packet camouflage configuration.
    pub tcp: Option<Vec<RootOutboundsItemStreamSettingsFinalmaskTcpItem>>,
}

/// QUIC final-mask parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsFinalmaskQuicParams {
    /// Enables debug mode.
    pub debug: bool,
    /// Congestion-control algorithm.
    pub congestion: String,
}

/// TCP final-mask item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsFinalmaskTcpItem {
    /// Mask type.
    pub r#type: String,
    /// Mask parameters.
    pub settings: RootOutboundsItemStreamSettingsFinalmaskTcpItemSettings,
}

/// TCP final-mask parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsFinalmaskTcpItemSettings {
    /// Delay parameter.
    pub delay: String,
    /// Packet length parameter.
    pub length: String,
    /// Packet pattern.
    pub packets: String,
}

/// WebSocket transport settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsWsSettings {
    /// WebSocket path.
    pub path: String,
    /// Host header.
    pub host: String,
    /// Additional HTTP headers.
    pub headers: RootOutboundsItemStreamSettingsWsSettingsHeaders,
}

/// Additional WebSocket headers.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsWsSettingsHeaders {}

/// XHTTP transport settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsXhttpSettings {
    /// XHTTP mode.
    pub mode: String,
    /// HTTP host.
    pub host: String,
    /// Request path.
    pub path: String,
    /// Additional XHTTP options.
    pub extra: RootOutboundsItemStreamSettingsXhttpSettingsExtra,
}

/// Additional XHTTP options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsXhttpSettingsExtra {
    /// Multiplexing configuration.
    pub xmux: RootOutboundsItemStreamSettingsXhttpSettingsExtraXmux,
    /// Padding byte range.
    #[serde(rename = "xPaddingBytes")]
    pub x_padding_bytes: String,
}

/// XHTTP multiplexing options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemStreamSettingsXhttpSettingsExtraXmux {
    /// Maximum reuse count range.
    #[serde(rename = "cMaxReuseTimes")]
    pub c_max_reuse_times: String,
    /// Maximum concurrency range.
    #[serde(rename = "maxConcurrency")]
    pub max_concurrency: String,
    /// Maximum number of connections.
    #[serde(rename = "maxConnections")]
    pub max_connections: i64,
    /// HTTP keep-alive period.
    #[serde(rename = "hKeepAlivePeriod")]
    pub h_keep_alive_period: i64,
    /// Maximum reusable connection lifetime.
    #[serde(rename = "hMaxReusableSecs")]
    pub h_max_reusable_secs: String,
}

/// Multiplexing options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOutboundsItemMux {
    /// Enables multiplexing.
    pub enabled: bool,
    /// Multiplexing concurrency.
    pub concurrency: i64,
    /// XUDP concurrency.
    #[serde(rename = "xudpConcurrency")]
    pub xudp_concurrency: i64,
    /// XUDP behavior for UDP/443.
    #[serde(rename = "xudpProxyUDP443")]
    pub xudp_proxy_udp443: String,
}
