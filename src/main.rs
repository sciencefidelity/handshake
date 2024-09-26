use std::io;
use std::time::Duration;

use pea2pea::{protocols::Handshake, Pea2Pea};
use tokio::time::sleep;

use handshake::{telemetry::init_subscriber, HelloNode};
use tracing::{event, instrument, level_filters::LevelFilter, Level};

#[tokio::main]
async fn main() -> io::Result<()> {
    init_subscriber(LevelFilter::TRACE);
    run().await;
    Ok(())
}

#[instrument]
async fn run() {
    event!(Level::INFO, "Starting up");

    // Create two test nodes.
    let initiator = HelloNode::new("initiator");
    let responder = HelloNode::new("responder");

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
    event!(Level::INFO, "{responder_connections:?}");
}
