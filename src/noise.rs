use std::io;

use pea2pea::{protocols::Handshake, Connection, ConnectionSide, Pea2Pea};
use snow::Builder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

use crate::node;

/// A Noise Protocol Framework handshake using the XX pattern.
///
/// Both the initiator and responder possess static key pairs.
/// and the handshake pattern comprises three message patterns.
///
/// ```plaintext
/// XX:
///   -> e
///   <- e, ee, s, es
///   -> s, se
/// ```
///
pub async fn handshake_xx(
    node: &node::Hello,
    conn: &mut Connection,
    buffer: &mut [u8],
) -> io::Result<()> {
    // Get current connection side.
    let node_conn_side = !conn.side();
    // Borrow full TCP stream.
    let stream = node.borrow_stream(conn);
    // Initialize noise builder.
    let local_private_key = Builder::new(node.get_noise_params().clone())
        .local_private_key(node.get_keypair().private.as_ref());

    match node_conn_side {
        ConnectionSide::Initiator => {
            let mut noise = local_private_key.build_initiator().unwrap();

            // -> e
            let len = noise.write_message(&[], buffer).unwrap();
            stream.write_all(&buffer[..len]).await?;
            info!("{:?} sent e: (handshake pt 1/3)", node.node().name());

            // <- e, ee, s, es
            let len = stream.read(buffer).await?;
            noise.read_message(&buffer[..len], &mut []).unwrap();
            info!(
                "{:?} received e, ee, s, es: (handshake pt 2/3)",
                node.node().name()
            );

            // -> s, se
            let len = noise.write_message(&[], buffer).unwrap();
            stream.write_all(&buffer[..len]).await?;
            info!("{:?} sent s, se: (handshake pt 3/3)", node.node().name());
        }
        ConnectionSide::Responder => {
            let mut noise = local_private_key.build_responder().unwrap();

            // <- e
            let len = stream.read(buffer).await?;
            noise.read_message(&buffer[..len], &mut []).unwrap();
            info!("{:?} sent e: (handshake pt 1/3)", node.node().name());

            // -> e, ee, s, es
            let len = noise.write_message(&[], buffer).unwrap();
            stream.write_all(&buffer[..len]).await?;
            info!(
                "{:?} received e, ee, s, es: (handshake pt 3/3)",
                node.node().name()
            );

            // <- s, se
            let len = stream.read(buffer).await?;
            noise.read_message(&buffer[..len], &mut []).unwrap();
            info!("{:?} sent s, se: (handshake pt 3/3)", node.node().name());
        }
    }

    Ok(())
}
