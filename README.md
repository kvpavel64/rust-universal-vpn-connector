# vpn-client

Composable asynchronous Rust library for Windows x64 (and other platforms supported by
sing-box). It downloads and parses proxy subscriptions, starts a local proxy engine, and
exposes local SOCKS5 and HTTP proxy endpoints. It does not install a TUN adapter or change
system routing.

## Supported subscription protocols

- VLESS
- VMess
- Trojan
- Shadowsocks
- SOCKS5
- HTTP / HTTPS proxy
- Hysteria2

The default engine is **sing-box**. The application must obtain a compatible `sing-box.exe`
separately and set its path with `core_path`. The crate does not silently download or update
executables.

## Quick start

```rust,no_run
use vpn_client::VpnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vpn = VpnClient::builder()
        .subscription("https://example.org/subscription")
        .core_path("./sing-box.exe")
        .build().await?;

    vpn.start(0).await?;
    let client = vpn.http_client()?;
    let response = client.get("https://api.ipify.org").send().await?;
    println!("{}", response.text().await?);
    vpn.shutdown().await?;
    Ok(())
}
```

Default listeners:

| Protocol | Address |
|---|---|
| SOCKS5 | `127.0.0.1:10808` |
| HTTP CONNECT | `127.0.0.1:10809` |

`http_client()` routes through SOCKS5 and `http_client_via_http_proxy()` routes through
HTTP. For other applications, configure their proxy manually to one of the local addresses.

## Replaceable components

Public extension traits are in `vpn_client::contracts`:

- `SubscriptionSource` — fetches subscription documents.
- `SubscriptionParser` — normalizes documents into `Node` values.
- `ProxyEngine` / `RunningEngine` — starts and owns the proxy runtime.
- `MessageBus` / `Mailbox` — transports commands and replies between components.

Defaults are `HttpSubscriptionSource`, `DefaultSubscriptionParser`, `SingBoxEngine`, and
`TokioMpscBus`. All can be replaced through `VpnClient::builder()`; see
[`examples/custom_components.rs`](examples/custom_components.rs).

## Message transport abstraction

The client facade sends command envelopes through `MessageBus`. The default
`TokioMpscBus` is in-process, bounded, non-durable, and at-most-once. A RabbitMQ/NATS/Redis
Streams adapter can implement the same `MessageBus` and `Mailbox` contracts. The message
envelope preserves message ID, sender, reply address, correlation ID, and JSON payload.

Broker-specific guarantees (durability, acknowledgement, ordering, redelivery, reconnects)
remain the adapter's responsibility. Messaging failures have dedicated error variants (duplicate
mailbox, unknown destination, closed mailbox, invalid address/capacity, correlation mismatch,
and request timeout); actor domain failures are mapped back to typed client errors where possible. See [`docs/architecture.md`](docs/architecture.md)
for the component map, message flow, and broker integration guidance.

## Build and test

```powershell
cargo test --all-targets
cargo test --doc
cargo build --release --target x86_64-pc-windows-msvc
```

The unit and integration tests do not require a real subscription or sing-box executable.
The basic example does require both. The custom-components example uses an embedded fixture
for subscription loading but still requires an engine executable only if you call `start`.

## Current limitations

- Share-link parsing covers common URI formats; provider-specific YAML, Clash, and
  subscription dialects may need custom `SubscriptionParser` implementations.
- No automatic node health checks, latency ranking, failover, or subscription refresh timer.
- `add_subscription` replaces the node list. An already running engine keeps the config
  with which it was started.
- The Tokio bus is process-local and non-durable. A broker adapter is an extension point,
  not included in this crate.
- The code has not been compiled in the authoring environment because Rust/Cargo are not
  installed there. Run the commands above before treating it as a release build.

## License

MIT. See [LICENSE](LICENSE).

## Typed identifiers and errors (0.3.0)

The messaging API uses dedicated newtypes rather than interchangeable strings:

```rust
use serde_json::json;
use vpn_client::{ComponentAddress, CorrelationId, MailboxCapacity, Message};

fn typed_message_example() -> Result<(), vpn_client::Error> {
    let sender = ComponentAddress::new("subscription-manager")?;
    let mut message = Message::new(sender, json!({ "event": "subscription_loaded" }));
    message.correlation_id = Some(CorrelationId::from_message_id(message.id));
    let _capacity = MailboxCapacity::new(128)?;
    let _ = message;
    Ok(())
}
```

`MessageId` is a UUIDv6 newtype. `CorrelationId` is a separate UUIDv6 newtype for
request/reply matching. IDs use a process-local random node identifier rather than the host
MAC address; UUIDv6 exposes its timestamp component. `ComponentAddress` validates construction and deserialization;
`MailboxCapacity` cannot represent zero. `MessagePayload` distinguishes the opaque JSON
body from transport metadata.

Messaging errors are represented by specific `Error` variants such as
`DuplicateMailbox`, `MailboxNotFound`, `MailboxClosed`, `InvalidMailboxAddress`,
`InvalidMailboxCapacity`, `MessageCorrelationMismatch`, and `MessageTimeout`. Known actor
failures are mapped back to their domain variants (for example `AlreadyRunning` and
`NodeIndex`); unexpected remote engine failures retain their diagnostic text in
`RemoteEngineFailure` or `RemoteActorFailure`.

## Xray JSON configuration documents

The subscription domain also exposes a strongly typed model for Xray-style JSON
configuration exports. It is deliberately separate from the normalized sing-box
`Node` representation:

```rust
use vpn_client::prelude::xray_config_prelude::*;

let json = r#"{"dns":{"servers":[],"queryStrategy":"UseIP"},"inbounds":[],"log":{"loglevel":"warning"},"outbounds":[],"remarks":"demo","routing":{"domainStrategy":"AsIs","domainMatcher":"hybrid","rules":[]}}"#;
let configs = Nodes::parse(json)?;
for config in configs {
    println!("{}", config.remarks);
}
# Ok::<(), vpn_client::Error>(())
```

A single root object and an array of root objects are accepted. Local files can be
loaded with `Nodes::read_file(path)`. Parsing and file I/O return domain errors; the
API does not use `unwrap()` internally. The schema currently models the Xray fields
provided by the project integration, including inbounds, outbounds, transport/TLS,
routing, balancers, and burst observatory settings.

## Prelude modules

`vpn_client::prelude` groups exports by domain. For example:

- `client_prelude` — client facade and builder;
- `contracts_prelude` — replaceable component contracts;
- `subscription_prelude` — normalized subscription domain types;
- `subscription_adapters_prelude` — default subscription adapters;
- `xray_config_prelude` — Xray JSON schema;
- `types_prelude` and `error_prelude` — shared value and error types.

Internal modules use the same domain-scoped prelude convention to avoid long, scattered
crate-root import lists.
