# Architecture and extension points

The crate separates domain policy from infrastructure. Public contracts are exported from
`vpn_client::contracts` and re-exported at the crate root where useful.

| Domain | Contract | Default implementation |
|---|---|---|
| Subscription acquisition | `SubscriptionSource` | `HttpSubscriptionSource` |
| Subscription parsing | `SubscriptionParser` | `DefaultSubscriptionParser` |
| Proxy runtime | `ProxyEngine`, `RunningEngine` | `SingBoxEngine` |
| Component messaging | `MessageBus`, `Mailbox` | `TokioMpscBus` |

## Message flow

The public `VpnClient` facade sends serialized command envelopes to a manager actor. The
actor owns the `RunningEngine` handle and handles `start`, `stop`, `status`, `replace_nodes`,
and `shutdown` commands. Subscription acquisition and parsing are performed by their
replaceable contracts; the normalized node list is then sent to the manager as a message.
The manager invokes the selected `ProxyEngine` through its trait contract. This keeps the
process handle local to its owner; a future engine actor can be split out without changing
the public facade or the message envelope. Request/reply messages carry both `reply_to` and
`correlation_id`. The bus transports opaque JSON payloads and does not know domain command
semantics.

`TokioMpscBus` is an in-process, bounded, at-most-once transport. A successful send means
the destination channel accepted the message; it does not mean that the destination has
processed it. It is not durable and is not suitable for cross-process delivery.

## Replacing Tokio MPSC with RabbitMQ

Implement `MessageBus::register` by declaring or attaching to a queue for the supplied
logical address and returning a `Mailbox` implementation. The mailbox implementation must:

1. Preserve `Message.id`, `sender`, `reply_to`, `correlation_id`, and `payload`.
2. Route `send(destination, message)` to the queue associated with `destination`.
3. Implement `recv` with the broker's chosen acknowledgement policy.
4. Define reconnection, queue durability, ordering, redelivery, and duplicate-delivery
   behavior explicitly.
5. Avoid claiming exactly-once delivery unless the adapter can actually guarantee it.

A typical RabbitMQ mapping is one durable queue per long-lived actor address, with the
address as a routing key. Reply mailboxes are short-lived and should use exclusive,
auto-delete queues. The adapter may encode the envelope as JSON or another stable wire
format. Do not put broker-specific types in domain contracts.

The current client creates a short-lived reply mailbox for each request. A broker adapter
must support this pattern or provide an equivalent correlation-based reply mechanism.
For production deployments, consider a mailbox TTL and cleanup policy for abandoned
requests.

## Replacing other components

Use the builder methods:

- `subscription_source(Arc<dyn SubscriptionSource>)`
- `subscription_parser(Arc<dyn SubscriptionParser>)`
- `proxy_engine(Arc<dyn ProxyEngine>)`
- `message_bus(Arc<dyn MessageBus>)`

The engine contract receives an `EngineConfig` containing the executable path, native
configuration, listener addresses, and startup deadline. Its returned `RunningEngine`
handle owns the running resource and must stop it in `stop` or on drop.

## Testing strategy

- Unit tests cover URI normalization, Base64-wrapped subscriptions, sing-box JSON, invalid
  documents, message routing, and generated inbound configuration.
- `tests/architecture.rs` uses fake source/parser/engine adapters to exercise the facade and
  actor without network access or a real proxy executable.
- `cargo test --all-targets` runs unit, integration, and example compilation tests.
- `cargo test --doc` runs the crate-level usage example.

The parser deliberately supports common share-link forms, not every vendor-specific
subscription dialect. For production use, add fixtures from the subscription providers you
intend to support and test the emitted sing-box configuration against the exact sing-box
version you ship.


## Typed message envelope

The transport contract deliberately avoids raw strings for semantically distinct values:

- `MessageId` is a UUIDv6 generated for each message. UUIDv6 preserves time ordering while
  retaining UUID uniqueness semantics. A random 48-bit node identifier is generated once per
  process; the host MAC address is not used. As with all timestamp-based UUIDs, the creation
  time is observable from the identifier.
- `CorrelationId` identifies a request/response exchange and is distinct from a message ID.
- `ComponentAddress` is a validated logical mailbox address. It rejects empty values,
  surrounding whitespace, control characters, and values longer than 255 bytes.
- `MailboxCapacity` guarantees a positive bounded queue capacity.
- `MessagePayload` marks the opaque JSON body and keeps it distinct from routing metadata.

A broker adapter must serialize these newtypes without changing their values. The Tokio
adapter maps typed addresses to bounded MPSC senders.

## Error model

The public `Error` enum distinguishes mailbox registry poisoning, duplicate registration,
unknown destinations, closed destinations, invalid addresses/capacities, correlation mismatch,
request timeout, actor termination, invalid node selection, and engine lifecycle failures.
Actor responses carry a stable error kind so known domain failures (for example, `AlreadyRunning`
and `NodeIndex`) remain typed after crossing the message boundary. Unknown engine failures are
returned as `RemoteEngineFailure` or `RemoteActorFailure` with the original diagnostic string.
