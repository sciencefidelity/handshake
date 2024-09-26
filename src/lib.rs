pub mod telemetry;

use std::io;

use pea2pea::{protocols::Handshake, Config, Connection, Node, Pea2Pea};

pub struct PeerData {}

#[derive(Clone)]
pub struct HelloNode {
    node: Node,
}

impl HelloNode {
    pub fn new(name: &str) -> Self {
        let config = Config {
            name: Some(name.to_owned()),
            ..Default::default()
        };
        let node = Node::new(config);
        Self { node }
    }
}

impl Pea2Pea for HelloNode {
    fn node(&self) -> &Node {
        &self.node
    }
}

impl Handshake for HelloNode {
    async fn perform_handshake(&self, conn: Connection) -> io::Result<Connection> {
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use crate::telemetry::init_subscriber;
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing::level_filters::LevelFilter;

    use super::*;

    #[tokio::test]
    async fn node_handshake() {
        init_subscriber(LevelFilter::TRACE);

        // Create two test nodes.
        let initiator = HelloNode::new("dialer");
        let responder = HelloNode::new("listener");

        // Enable handshake for both test nodes.
        for node in [&initiator, &responder] {
            node.enable_handshake().await;
        }

        // Start responder listening for incoming connections. Store responder address.
        let responder_addr = responder.node().toggle_listener().await.unwrap().unwrap();

        // Initiator attempts to connect to responder address.
        initiator.node().connect(responder_addr).await.unwrap();

        // Wait for connection to establish.
        sleep(Duration::from_millis(100)).await;

        let responder_connections = responder.node().connected_addrs();
        println!("{responder_connections:?}");
    }
}
