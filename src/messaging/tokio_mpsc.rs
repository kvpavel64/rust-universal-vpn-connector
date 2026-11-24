//! In-process bounded message bus implemented with Tokio MPSC channels.
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

use crate::prelude::error_prelude::*;
use crate::prelude::messaging_prelude::*;
use crate::prelude::types_prelude::*;

/// Default in-process broker with bounded, at-most-once, non-durable delivery.
#[derive(Clone, Default)]
pub struct TokioMpscBus {
    routes: Arc<Mutex<HashMap<ComponentAddress, mpsc::Sender<Message>>>>,
}

#[async_trait]
impl MessageBus for TokioMpscBus {
    async fn register(
        &self,
        address: ComponentAddress,
        capacity: MailboxCapacity,
    ) -> Result<Box<dyn Mailbox>> {
        let mut routes = self
            .routes
            .lock()
            .map_err(|_| Error::MailboxRegistryPoisoned)?;
        if routes.contains_key(&address) {
            return Err(Error::DuplicateMailbox(address));
        }
        let (tx, rx) = mpsc::channel(capacity.get());
        routes.insert(address.clone(), tx);
        Ok(Box::new(TokioMpscMailbox {
            address,
            routes: self.routes.clone(),
            rx,
        }))
    }
}

struct TokioMpscMailbox {
    address: ComponentAddress,
    routes: Arc<Mutex<HashMap<ComponentAddress, mpsc::Sender<Message>>>>,
    rx: mpsc::Receiver<Message>,
}

#[async_trait]
impl Mailbox for TokioMpscMailbox {
    fn address(&self) -> &ComponentAddress {
        &self.address
    }

    async fn send(&mut self, destination: &ComponentAddress, message: Message) -> Result<()> {
        let sender = self
            .routes
            .lock()
            .map_err(|_| Error::MailboxRegistryPoisoned)?
            .get(destination)
            .cloned()
            .ok_or_else(|| Error::MailboxNotFound(destination.clone()))?;
        sender
            .send(message)
            .await
            .map_err(|_| Error::MailboxClosed(destination.clone()))
    }

    async fn recv(&mut self) -> Result<Option<Message>> {
        Ok(self.rx.recv().await)
    }
}

impl Drop for TokioMpscMailbox {
    fn drop(&mut self) {
        if let Ok(mut routes) = self.routes.lock() {
            routes.remove(&self.address);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn address(value: &str) -> ComponentAddress {
        ComponentAddress::new(value).unwrap()
    }

    #[tokio::test]
    async fn routes_messages_between_named_mailboxes() {
        let bus = TokioMpscBus::default();
        let mut sender = bus
            .register(address("sender"), MailboxCapacity::new(4).unwrap())
            .await
            .unwrap();
        let mut receiver = bus
            .register(address("receiver"), MailboxCapacity::new(4).unwrap())
            .await
            .unwrap();
        sender
            .send(
                &address("receiver"),
                Message::new(address("sender"), json!({"kind":"ping"})),
            )
            .await
            .unwrap();
        let message = receiver.recv().await.unwrap().unwrap();
        assert_eq!(message.payload.as_value()["kind"], "ping");
    }

    #[test]
    fn envelope_round_trips_with_typed_identifiers() {
        let mut message = Message::new(address("component-a"), json!({"command":"ping"}));
        message.reply_to = Some(address("component-a-replies"));
        message.correlation_id = Some(crate::CorrelationId::from_message_id(message.id));
        let encoded = serde_json::to_string(&message).unwrap();
        let decoded: Message = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.id, message.id);
        assert_eq!(decoded.id.as_uuid().get_version_num(), 6);
        assert_eq!(decoded.sender, address("component-a"));
        assert_eq!(decoded.reply_to, Some(address("component-a-replies")));
        assert_eq!(decoded.correlation_id, message.correlation_id);
        assert_eq!(decoded.payload.as_value()["command"], "ping");
    }

    #[tokio::test]
    async fn rejects_duplicate_addresses_and_unknown_destinations() {
        let bus = TokioMpscBus::default();
        let _first = bus
            .register(address("same"), MailboxCapacity::new(1).unwrap())
            .await
            .unwrap();
        assert!(matches!(
            bus.register(address("same"), MailboxCapacity::new(1).unwrap())
                .await,
            Err(Error::DuplicateMailbox(_))
        ));
        let mut sender = bus
            .register(address("sender"), MailboxCapacity::new(1).unwrap())
            .await
            .unwrap();
        assert!(matches!(
            sender
                .send(
                    &address("missing"),
                    Message::new(address("sender"), json!({}))
                )
                .await,
            Err(Error::MailboxNotFound(_))
        ));
    }

    #[test]
    fn typed_payload_round_trips() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Command {
            sequence: u64,
        }
        let payload = crate::MessagePayload::from_serializable(&Command { sequence: 42 }).unwrap();
        let decoded: Command = payload.deserialize().unwrap();
        assert_eq!(decoded, Command { sequence: 42 });
    }

    #[test]
    fn rejects_invalid_addresses_and_zero_capacity() {
        assert!(matches!(
            ComponentAddress::new(""),
            Err(Error::InvalidMailboxAddress(_))
        ));
        assert!(matches!(
            MailboxCapacity::new(0),
            Err(Error::InvalidMailboxCapacity)
        ));
        assert!(serde_json::from_str::<ComponentAddress>("\"bad\\naddress\"").is_err());
    }
}
