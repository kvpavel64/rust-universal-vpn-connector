//! Top-level Xray-core configuration document.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    inbound::RootInboundsItem, observatory::RootBurstObservatory, outbound::RootOutboundsItem,
    routing::RootRouting,
};

/// Xray outbound collection used by subscription consumers.
pub type Nodes = Vec<RootOutboundsItem>;

/// Result of reading a subscription configuration file.
pub type FileReadResult = std::io::Result<String>;

/// Result of deserializing a JSON configuration document.
pub type JsonResult<T> = std::result::Result<T, serde_json::Error>;

/// Flexible Xray DNS configuration.
///
/// Xray has a broad, version-dependent DNS schema. Keeping this value-backed
/// preserves fields unknown to this crate instead of rejecting valid configs.
pub type RootDns = Value;

/// Flexible Xray logging configuration.
pub type RootLog = Value;

/// Xray-core JSON configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Root {
    /// Optional logging configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log: Option<RootLog>,
    /// Optional DNS configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns: Option<RootDns>,
    /// Inbound listeners.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inbounds: Vec<RootInboundsItem>,
    /// Outbound proxy definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outbounds: Vec<RootOutboundsItem>,
    /// Optional routing configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<RootRouting>,
    /// Optional burst observatory configuration.
    #[serde(
        rename = "burstObservatory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub burst_observatory: Option<RootBurstObservatory>,
    /// Additional Xray settings not modeled by this crate.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
