#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
use std::time::Duration;

use pea2pea::{protocols::Handshake, Pea2Pea};
use tokio::time::sleep;

use handshake::{node, noise, telemetry::init_subscriber};
use tracing::{info, level_filters::LevelFilter};

#[tokio::main]
async fn main() -> Result<(), noise::Error> {
    init_subscriber(LevelFilter::TRACE);
    run().await?;
    Ok(())
}

async fn run() -> Result<(), noise::Error> {
    info!("starting up");

    // Create two test nodes.
    let initiator = node::Hello::new("initiator")?;
    let responder = node::Hello::new("responder")?;

    // Enable handshake for both test nodes.
    tokio::join!(initiator.enable_handshake(), responder.enable_handshake());

    // Start responder listening for incoming connections. Store responder address.
    let responder_addr = responder.node().toggle_listener().await.unwrap().unwrap();

    // Initiator attempts to connect to responder address.
    initiator.node().connect(responder_addr).await.unwrap();

    // Wait for connection to establish.
    sleep(Duration::from_millis(100)).await;

    let initiator_attr = responder.node().connected_addrs()[0];

    // Disconnect everyone from the stream.
    tokio::join!(
        initiator.node().disconnect(responder_addr),
        responder.node().disconnect(initiator_attr),
    );

    Ok(())
}
