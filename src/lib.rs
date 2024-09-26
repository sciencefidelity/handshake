#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unwrap_used
)]
pub mod telemetry;

use std::{io, sync::Arc};

use pea2pea::{protocols::Handshake, Config, Connection, ConnectionSide, Node, Pea2Pea};
use snow::{params::NoiseParams, Builder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

pub use telemetry::init_subscriber;

const MAX_MESSAGE_LEN: usize = 65535;

#[derive(Clone)]
struct StaticKey(Arc<snow::Keypair>);

impl StaticKey {
    pub fn generate(noise_params: &NoiseParams) -> Self {
        let static_key = Arc::new(
            Builder::new(noise_params.clone())
                .generate_keypair()
                .unwrap(),
        );
        Self(static_key)
    }
}

#[derive(Clone)]
pub struct HelloNode {
    node: Node,
    noise_params: NoiseParams,
    keypair: StaticKey,
}

impl HelloNode {
    pub fn new(name: &str) -> Self {
        // Build the node's configuration.
        let config = Config {
            name: Some(name.to_owned()),
            ..Default::default()
        };
        let node = Node::new(config);

        // Initialize Noise Protocol
        let noise_params = "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse().unwrap();

        // Generate static key
        let keypair = StaticKey::generate(&noise_params);

        Self {
            node,
            noise_params,
            keypair,
        }
    }
}

impl Pea2Pea for HelloNode {
    fn node(&self) -> &Node {
        &self.node
    }
}

impl Handshake for HelloNode {
    async fn perform_handshake(&self, mut conn: Connection) -> io::Result<Connection> {
        let mut buffer = [0u8; MAX_MESSAGE_LEN];

        // Get current connection side.
        let node_conn_side = !conn.side();
        // Borrow full TCP stream.
        let stream = self.borrow_stream(&mut conn);
        // Initialize noise builder.
        let local_private_key = Builder::new(self.noise_params.clone())
            .local_private_key(self.keypair.0.private.as_ref());

        match node_conn_side {
            // Bob's stream
            ConnectionSide::Initiator => {
                let mut noise = local_private_key.build_initiator().unwrap();

                // -> e
                let len = noise.write_message(&[], &mut buffer).unwrap();
                stream.write_all(&buffer[..len]).await.unwrap();
                info!("{:?} sent e: (handshake pt 1/3)", self.node().name());

                // <- e, ee, s, es
                let len = stream.read(&mut buffer).await.unwrap();
                noise.read_message(&buffer[..len], &mut []).unwrap();
                info!(
                    "{:?} received e, ee, s, es: (handshake pt 2/3)",
                    self.node().name()
                );

                // -> s, se
                let len = noise.write_message(&[], &mut buffer).unwrap();
                stream.write_all(&buffer[..len]).await.unwrap();
                info!("{:?} sent s, se: (handshake pt 3/3)", self.node().name());
            }
            // Alice's stream
            ConnectionSide::Responder => {
                let mut noise = local_private_key.build_responder().unwrap();

                // <- e
                let len = stream.read(&mut buffer).await.unwrap();
                noise.read_message(&buffer[..len], &mut []).unwrap();
                info!("{:?} sent e: (handshake pt 1/3)", self.node().name());

                // -> e, ee, s, es
                let len = noise.write_message(&[], &mut buffer).unwrap();
                stream.write_all(&buffer[..len]).await.unwrap();
                info!(
                    "{:?} received e, ee, s, es: (handshake pt 3/3)",
                    self.node().name()
                );

                // <- s, se
                let len = stream.read(&mut buffer).await.unwrap();
                noise.read_message(&buffer[..len], &mut []).unwrap();
                info!("{:?} sent s, se: (handshake pt 3/3)", self.node().name());
            }
        }

        info!("handshake complete!");
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing::level_filters::LevelFilter;

    use super::*;

    #[tokio::test]
    async fn node_handshake() {
        init_subscriber(LevelFilter::TRACE);

        // Create two test nodes.
        let bob = HelloNode::new("initiator");
        let alice = HelloNode::new("responder");

        // Enable handshake for both test nodes.
        tokio::join!(bob.enable_handshake(), alice.enable_handshake(),);

        // Start Alice listening for incoming connections.
        // Store Alice's address to tell Bob.
        let alice_addr = alice.node().toggle_listener().await.unwrap().unwrap();

        // Bob attempts to connect to Alice's address.
        bob.node().connect(alice_addr).await.unwrap();

        // Waiting for a connection to establish.
        sleep(Duration::from_millis(100)).await;

        // Check that nodes both nodes have a connection.
        assert_eq!(bob.node().num_connected(), 1);
        assert_eq!(alice.node().num_connected(), 1);

        let bob_addr = alice.node().connected_addrs()[0];

        // Check that Bob and Alice are connected to each other.

        // Check that Bob and Alice's secrets match.

        // Disconnect everyone from the stream.
        tokio::join!(
            bob.node().disconnect(alice_addr),
            alice.node().disconnect(bob_addr)
        );
    }
}
