//! Strongly typed identifiers and values shared by the public API.
use std::{fmt, num::NonZeroUsize, sync::OnceLock};
use crate::prelude::error_prelude::*;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// Generates UUIDv6 values using a process-local random node identifier.
pub(crate) fn new_uuid_v6() -> Uuid {
    static NODE_ID: OnceLock<[u8; 6]> = OnceLock::new();
    let node_id = NODE_ID.get_or_init(|| {
        let bytes = Uuid::new_v4().into_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
    });
    Uuid::now_v6(node_id)
}

/// Globally unique message identifier generated as a time-ordered UUID version 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Generates a new time-ordered UUIDv6 message identifier.
    pub fn new() -> Self { Self(new_uuid_v6()) }
    /// Returns the underlying UUID.
    pub const fn as_uuid(&self) -> &Uuid { &self.0 }
}
impl Default for MessageId { fn default() -> Self { Self::new() } }
impl fmt::Display for MessageId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }

/// Identifier shared by a request and its response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CorrelationId(Uuid);
impl CorrelationId {
    /// Generates a new correlation identifier.
    pub fn new() -> Self { Self(new_uuid_v6()) }
    /// Creates a correlation identifier from a message identifier.
    pub const fn from_message_id(id: MessageId) -> Self { Self(id.0) }
    /// Returns the underlying UUID.
    pub const fn as_uuid(&self) -> &Uuid { &self.0 }
}
impl Default for CorrelationId { fn default() -> Self { Self::new() } }
impl fmt::Display for CorrelationId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }

/// Validated logical address of a component mailbox.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ComponentAddress(String);
impl ComponentAddress {
    /// Creates an address. Empty values, control characters, and values over 255 bytes are rejected.
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if value.trim().is_empty() || value != value.trim() || value.len() > 255 || value.chars().any(char::is_control) {
            return Err(Error::InvalidMailboxAddress(value));
        }
        Ok(Self(value))
    }
    /// Returns the address as a string slice.
    pub fn as_str(&self) -> &str { &self.0 }
}
impl<'de> Deserialize<'de> for ComponentAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}
impl fmt::Display for ComponentAddress { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }

/// Positive capacity requested for a mailbox queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MailboxCapacity(NonZeroUsize);
impl MailboxCapacity {
    /// Creates a non-zero queue capacity.
    pub fn new(value: usize) -> Result<Self, Error> {
        NonZeroUsize::new(value).map(Self).ok_or(Error::InvalidMailboxCapacity)
    }
    /// Returns the queue capacity.
    pub const fn get(self) -> usize { self.0.get() }
}
impl Default for MailboxCapacity { fn default() -> Self { Self(NonZeroUsize::new(64).expect("64 is nonzero")) } }
