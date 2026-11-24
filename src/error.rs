//! Error types shared by the public API and default adapters.
use thiserror::Error;
use crate::prelude::types_prelude::*;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the VPN client and its default components.
#[derive(Debug, Error)]
pub enum Error {
    /// An HTTP subscription request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// A URL could not be parsed.
    #[error("URL parse failed: {0}")]
    Url(#[from] url::ParseError),
    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON encoding or decoding failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// No supported nodes were found in a subscription.
    #[error("subscription contains no supported nodes")]
    EmptySubscription,
    /// A node URI is malformed or unsupported.
    #[error("unsupported or malformed proxy URI: {0}")]
    InvalidNode(String),
    /// The requested node index does not exist.
    #[error("node index {0} is out of range")]
    NodeIndex(usize),
    /// The external core exited before its listeners became ready.
    #[error("proxy engine exited before local listeners became ready")]
    CoreExited,
    /// The local listeners did not become ready before the deadline.
    #[error("local proxy did not become ready before timeout")]
    StartupTimeout,
    /// The client is not currently connected.
    #[error("VPN client is not running")]
    NotRunning,
    /// The client is already connected.
    #[error("VPN client is already running")]
    AlreadyRunning,
    /// The configured engine executable does not exist.
    #[error("invalid proxy engine executable path")]
    InvalidCorePath,
    /// Local proxy listener addresses must be distinct loopback addresses with nonzero ports.
    #[error("SOCKS5 and HTTP listeners must use distinct loopback addresses and nonzero ports")]
    InvalidProxyAddress,
    /// The in-process mailbox registry lock was poisoned.
    #[error("message bus registry lock is poisoned")]
    MailboxRegistryPoisoned,
    /// A mailbox with the same logical address is already registered.
    #[error("mailbox `{0}` is already registered")]
    DuplicateMailbox(ComponentAddress),
    /// The requested mailbox address is not registered.
    #[error("mailbox `{0}` is not registered")]
    MailboxNotFound(ComponentAddress),
    /// The destination mailbox has been closed.
    #[error("mailbox `{0}` is closed")]
    MailboxClosed(ComponentAddress),
    /// A logical mailbox address is empty, too long, or contains control characters.
    #[error("invalid mailbox address: {0:?}")]
    InvalidMailboxAddress(String),
    /// A mailbox queue capacity must be greater than zero.
    #[error("mailbox capacity must be greater than zero")]
    InvalidMailboxCapacity,
    /// A response did not match the request correlation identifier.
    #[error("message correlation mismatch: expected {expected}, received {actual:?}")]
    MessageCorrelationMismatch {
        /// Correlation identifier assigned to the request.
        expected: crate::CorrelationId,
        /// Correlation identifier returned by the actor, if any.
        actual: Option<crate::CorrelationId>,
    },
    /// A remote engine adapter failed and returned its diagnostic message.
    #[error("remote proxy engine failure: {0}")]
    RemoteEngineFailure(String),
    /// A remote actor rejected a malformed or unsupported control message.
    #[error("remote actor failure: {0}")]
    RemoteActorFailure(String),
    /// The actor received a command that is not part of its command contract.
    #[error("unknown actor command: {0}")]
    UnknownActorCommand(String),
    /// Reading a local subscription file failed.
    #[error("failed to read subscription file: {0}")]
    SubscriptionFileRead(#[source] std::io::Error),
    /// An Xray JSON configuration document is invalid.
    #[error("invalid Xray configuration JSON: {0}")]
    XrayConfigJson(#[source] serde_json::Error),
    /// A request/reply operation exceeded its deadline.
    #[error("message request timed out")]
    MessageTimeout,
    /// The client actor has stopped receiving commands.
    #[error("client actor has stopped")]
    ActorStopped,
}
