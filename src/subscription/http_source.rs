//! Default HTTPS subscription downloader.
use async_trait::async_trait;

use crate::prelude::contracts_prelude::*;
use crate::prelude::error_prelude::*;

/// HTTP(S)-based subscription source backed by `reqwest`.
#[derive(Clone)]
pub struct HttpSubscriptionSource {
    client: reqwest::Client,
}

impl Default for HttpSubscriptionSource {
    fn default() -> Self { Self { client: reqwest::Client::new() } }
}

impl HttpSubscriptionSource {
    /// Creates a source with a caller-configured HTTP client.
    pub fn new(client: reqwest::Client) -> Self { Self { client } }
}

#[async_trait]
impl SubscriptionSource for HttpSubscriptionSource {
    async fn fetch(&self, location: &str) -> Result<String> {
        Ok(self.client.get(location).send().await?.error_for_status()?.text().await?)
    }
}
