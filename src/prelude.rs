//! Convenient, domain-scoped re-exports for crate modules and downstream users.

/// Client facade and builder exports.
pub mod client_prelude { pub use crate::client::{ClientBuilder, ProxyKind, VpnClient}; }
/// Public component contracts.
pub mod contracts_prelude { pub use crate::contracts::{EngineConfig, Mailbox, Message, MessageBus, MessagePayload, ProxyEngine, RunningEngine, SubscriptionParser, SubscriptionSource}; }
/// Error types.
pub mod error_prelude { pub use crate::error::{Error, Result}; }
/// Engine adapter contracts (does not import the adapter currently being implemented).
pub mod engine_prelude { pub use crate::contracts::{EngineConfig, ProxyEngine, RunningEngine}; }
/// Default sing-box adapter.
pub mod sing_box_prelude { pub use crate::engine::SingBoxEngine; }
/// Messaging contracts.
pub mod messaging_prelude { pub use crate::contracts::{Mailbox, Message, MessageBus, MessagePayload}; }
/// Default Tokio MPSC transport.
pub mod tokio_mpsc_prelude { pub use crate::messaging::TokioMpscBus; }
/// Core subscription domain types.
pub mod subscription_prelude { pub use crate::subscription::{Node, Protocol}; }
/// Default subscription adapters.
pub mod subscription_adapters_prelude { pub use crate::subscription::{DefaultSubscriptionParser, HttpSubscriptionSource}; }
/// Strongly typed shared values.
pub mod types_prelude { pub use crate::types::{ComponentAddress, CorrelationId, MailboxCapacity, MessageId}; }
/// Xray JSON configuration schema.
pub mod xray_config_prelude { pub use crate::subscription::xray_config::*; }
/// Types referenced by the Xray root document (excludes `Root` itself).
pub mod xray_root_prelude {
    pub use crate::subscription::xray_config::{RootBurstObservatory, RootInboundsItem, RootOutboundsItem, RootRouting};
}
