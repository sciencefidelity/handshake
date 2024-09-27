# handshake

## A P2P node handshake

This is a P2P node handshake based on the node implementation [pea2pea](https://github.com/ljedrz/pea2pea/tree/master). It uses a key exchange strategy based on the [Noise Protocol Framework](http://www.noiseprotocol.org) using [snow](https://github.com/mcginty/snow/tree/main) to make the handshake using a simple version of the [XX pattern](http://www.noiseprotocol.org/noise.html#handshake-pattern-basics) where the initiator and the responder both hold a static key pair. The exchange comprises three message patterns:

```
XX:
  -> e
  <- e, ee, s, es
  -> s, se
```

I've kept the procedure as minimal as possible, only mandatory parameters are exchanged and the connection is closed immediately after the exchange has completed.

### Noise Protocol Setup

We use the `Noise_XX_25519_ChaChaPoly_BLAKE2s` protocol, which defines:

- `XX`: A handshake pattern where both parties (initiator and responder) are anonymous initially.
- `25519`: Elliptic curve for key exchange (X25519).
- `ChaChaPoly`: Symmetric encryption for messages (ChaCha20-Poly1305).
- `BLAKE2s`: A cryptographic hash function for key derivation.

### Handshake Process

- Stage 1: The initiator sends the first message (an empty payload).
- Stage 2: The responder reads the first message, processes it, and sends a second message (also an empty payload).
- Stage 3: The initiator processes the second message and sends a third one, completing the handshake.
### Testing

To run the test suite run `cargo test`:

```shell
cargo test
```

To run a single exchange with logs run `cargo run`:

```shell
cargo run
```

You can also run the tests with logs by appending the `LOG_LEVEL` environment variable. Any log level in defined by [`tracing`](https://docs.rs/tracing/latest/tracing/struct.Level.html#implementations) in lowercase will work: `error`, `warn`, `info`, `debug`, `trace`.

```shell
LOG_LEVEL=info cargo test
```

