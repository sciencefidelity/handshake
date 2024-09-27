#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unwrap_used
)]
pub mod node;
pub mod noise;
pub mod telemetry;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pea2pea::{protocols::Handshake, Pea2Pea};
    use telemetry::init_subscriber;
    use tokio::time::sleep;
    use tracing::level_filters::LevelFilter;

    use super::*;

    #[tokio::test]
    async fn node_handshake() {
        init_subscriber(LevelFilter::TRACE);

        // Create two test nodes.
        let bob = node::Hello::new("initiator");
        let alice = node::Hello::new("responder");

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
