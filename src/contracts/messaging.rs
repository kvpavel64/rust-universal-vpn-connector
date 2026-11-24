//! Broker-neutral message transport contracts and strongly typed envelopes.
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::prelude::error_prelude::*;
use crate::prelude::types_prelude::*;

/// Opaque JSON payload carried by a [`Message`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessagePayload(Value);
impl MessagePayload {
    /// Wraps a JSON value as a message payload.
    pub fn new(value: Value) -> Self {
        Self(value)
    }
    /// Serializes a typed domain payload into the transport representation.
    pub fn from_serializable<T: Serialize>(
        value: &T,
    ) -> std::result::Result<Self, serde_json::Error> {
        serde_json::to_value(value).map(Self)
    }
    /// Deserializes the payload into a typed domain value.
    pub fn deserialize<T: DeserializeOwned>(&self) -> std::result::Result<T, serde_json::Error> {
        serde_json::from_value(self.0.clone())
    }
    /// Borrows the underlying JSON value.
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    /// Consumes the wrapper and returns the JSON value.
    pub fn into_value(self) -> Value {
        self.0
    }
}
impl From<Value> for MessagePayload {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

/// Serializable broker-neutral message envelope.
///
/// Transport adapters must preserve identifiers and addresses unchanged. Delivery
/// guarantees (durability, acknowledgement, ordering, redelivery) are adapter-specific.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// UUIDv6 identifier unique to this message.
    pub id: MessageId,
    /// Logical address of the sending component.
    pub sender: ComponentAddress,
    /// Optional logical address for a response.
    pub reply_to: Option<ComponentAddress>,
    /// Optional request/response correlation identifier.
    pub correlation_id: Option<CorrelationId>,
    /// Opaque domain payload.
    pub payload: MessagePayload,
}
impl Message {
    /// Creates a message with a newly generated UUIDv6 identifier.
    pub fn new(sender: ComponentAddress, payload: impl Into<MessagePayload>) -> Self {
        Self {
            id: MessageId::new(),
            sender,
            reply_to: None,
            correlation_id: None,
            payload: payload.into(),
        }
    }
}

/// A named endpoint that can send to and receive from other mailboxes.
#[async_trait]
pub trait Mailbox: Send {
    /// Returns this mailbox's logical address.
    fn address(&self) -> &ComponentAddress;
    /// Sends a message to a registered destination.
    async fn send(&mut self, destination: &ComponentAddress, message: Message) -> Result<()>;
    /// Waits for the next message. `None` means the endpoint has been closed.
    async fn recv(&mut self) -> Result<Option<Message>>;
}

/// Creates named mailboxes for component-to-component messaging.
///
/// Implementations should reject duplicate active addresses. A broker adapter is
/// responsible for queue declaration, serialization, reconnects, and delivery semantics.
#[async_trait]
pub trait MessageBus: Send + Sync + 'static {
    /// Registers a mailbox with a logical address and positive queue capacity.
    async fn register(
        &self,
        address: ComponentAddress,
        capacity: MailboxCapacity,
    ) -> Result<Box<dyn Mailbox>>;
}
