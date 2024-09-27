#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
pub mod node;
pub mod noise;
pub mod telemetry;

#[cfg(test)]
mod tests {
    use std::{io, str::FromStr, time::Duration};

    use noise::Error::{self, Io};
    use pea2pea::{connect_nodes, protocols::Handshake, Pea2Pea, Topology};
    use telemetry::init_subscriber;
    use tokio::time::sleep;
    use tracing::Level;

    use super::*;

    #[tokio::test]
    async fn node_handshake() -> Result<(), Error> {
        if let Ok(maybe_level) = std::env::var("LOG_LEVEL") {
            if let Ok(level) = Level::from_str(&maybe_level) {
                init_subscriber(level.into());
            }
        }

        // Create two test nodes.
        let bob = node::Hello::new("initiator")?;
        let alice = node::Hello::new("responder")?;

        // Enable handshake for both test nodes.
        tokio::join!(bob.enable_handshake(), alice.enable_handshake());

        // Start Alice listening for incoming connections.
        // Store Alice's address to tell Bob.
        let alice_addr = alice.node().toggle_listener().await.map_err(Io)?.unwrap();

        // Bob attempts to connect to Alice's address.
        bob.node().connect(alice_addr).await.map_err(Io)?;

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

        Ok(())
    }

    #[tokio::test]
    async fn linear_nodes_handshake() -> io::Result<()> {
        const NUM_NODES: usize = 7;
        const LAST_NODE: usize = NUM_NODES - 1;
        let nodes: [node::Hello; NUM_NODES] =
            std::array::from_fn(|i| node::Hello::new(format!("line-{i}").as_str()).unwrap());
        for node in &nodes {
            node.enable_handshake().await;
            node.node().toggle_listener().await?;
        }
        connect_nodes(&nodes, Topology::Line).await?;

        sleep(Duration::from_millis(100)).await;

        for (i, node) in nodes.iter().enumerate() {
            match i {
                0 | LAST_NODE => assert_eq!(node.node().connected_addrs().len(), 1),
                _ => assert_eq!(node.node().connected_addrs().len(), 2),
            }
        }

        disconnect_all_nodes(&nodes).await;
        Ok(())
    }

    #[tokio::test]
    async fn mesh_nodes_handshake() -> io::Result<()> {
        const NUM_NODES: usize = 7;
        let nodes: [node::Hello; NUM_NODES] =
            std::array::from_fn(|i| node::Hello::new(format!("mesh-{i}").as_str()).unwrap());
        for node in &nodes {
            node.enable_handshake().await;
            node.node().toggle_listener().await?;
        }
        connect_nodes(&nodes, Topology::Mesh).await?;

        sleep(Duration::from_millis(100)).await;

        for node in &nodes {
            assert_eq!(node.node().connected_addrs().len(), NUM_NODES - 1);
        }

        disconnect_all_nodes(&nodes).await;
        Ok(())
    }

    #[tokio::test]
    async fn star_nodes_handshake() -> io::Result<()> {
        const NUM_NODES: usize = 7;
        let nodes: [node::Hello; NUM_NODES] =
            std::array::from_fn(|i| node::Hello::new(format!("star-{i}").as_str()).unwrap());
        for node in &nodes {
            node.enable_handshake().await;
            node.node().toggle_listener().await?;
        }
        connect_nodes(&nodes, Topology::Star).await?;

        sleep(Duration::from_millis(100)).await;

        assert_eq!(nodes[0].node().connected_addrs().len(), NUM_NODES - 1);
        for node in nodes.iter().skip(1) {
            assert_eq!(node.node().connected_addrs().len(), 1);
        }

        disconnect_all_nodes(&nodes).await;
        Ok(())
    }

    async fn disconnect_all_nodes(nodes: &[node::Hello]) {
        for node in nodes {
            for addr in node.node().connected_addrs() {
                node.node().disconnect(addr).await;
            }
        }
    }
}
