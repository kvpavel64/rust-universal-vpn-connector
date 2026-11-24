/// Xray burst observatory configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootBurstObservatory {
    /// Ping measurement configuration.
    #[serde(rename = "pingConfig")]
    pub ping_config: RootBurstObservatoryPingConfig,
    /// Outbound tags to observe.
    #[serde(rename = "subjectSelector")]
    pub subject_selector: Vec<String>,
}

/// Ping measurement configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootBurstObservatoryPingConfig {
    /// Timeout per probe.
    pub timeout: String,
    /// Probe interval.
    pub interval: String,
    /// Number of samples.
    pub sampling: i64,
    /// Probe destination.
    pub destination: String,
    /// Connectivity-check URL.
    pub connectivity: String,
}
