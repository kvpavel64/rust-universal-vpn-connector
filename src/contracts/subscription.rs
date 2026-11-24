//! Contracts for obtaining and parsing subscription documents.
use async_trait::async_trait;

use crate::prelude::error_prelude::*;
use crate::prelude::subscription_prelude::*;

/// Fetches a subscription document from a source.
///
/// Implementations may read HTTP, a local file, a secret store, or another provider.
#[async_trait]
pub trait SubscriptionSource: Send + Sync + 'static {
    /// Fetches the raw subscription body.
    async fn fetch(&self, location: &str) -> Result<String>;
}

/// Converts a raw subscription document into normalized proxy nodes.
pub trait SubscriptionParser: Send + Sync + 'static {
    /// Parses a subscription into supported nodes.
    fn parse(&self, document: &str) -> Result<Vec<Node>>;
}
