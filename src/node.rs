use std::{io, sync::Arc};

use pea2pea::{protocols::Handshake, Config, Connection, Node, Pea2Pea};
use snow::{params::NoiseParams, Builder};
use tracing::info;

use crate::noise::handshake_xx;
use crate::noise::Error::{self, Snow};

/// Max length in bytes of a Noise Protocol message.
const MAX_MESSAGE_LEN: usize = 65535;

/// An asymmetric key pair for use as a [`StaticKey`].
#[derive(Clone)]
struct StaticKey(Arc<snow::Keypair>);

impl StaticKey {
    /// Generates a new asymmetric key pair.
    pub fn generate(noise_params: &NoiseParams) -> Result<Self, Error> {
        let static_key = Arc::new(
            Builder::new(noise_params.clone())
                .generate_keypair()
                .map_err(Snow)?,
        );
        Ok(Self(static_key))
    }
}

/// A very simple [Node].
#[derive(Clone)]
pub struct Hello {
    node: Node,
    noise_params: NoiseParams,
    static_key: StaticKey,
}

impl Hello {
    /// Creates a new [`HelloNode`] with the provided name.
    #[must_use]
    pub fn new(name: &str) -> Result<Self, Error> {
        // Build the node's configuration.
        let config = Config {
            name: Some(name.to_owned()),
            ..Default::default()
        };
        let node = Node::new(config);

        // Initialize Noise Protocol.
        let noise_params = "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse().map_err(Snow)?;
        // Generate static key pair.
        let static_key = StaticKey::generate(&noise_params)?;

        Ok(Self {
            node,
            noise_params,
            static_key,
        })
    }

    #[must_use]
    pub const fn get_noise_params(&self) -> &NoiseParams {
        &self.noise_params
    }

    #[must_use]
    pub fn get_keypair(&self) -> &snow::Keypair {
        self.static_key.0.as_ref()
    }
}

impl Pea2Pea for Hello {
    fn node(&self) -> &Node {
        &self.node
    }
}

impl Handshake for Hello {
    async fn perform_handshake(&self, mut conn: Connection) -> Result<Connection, io::Error> {
        let mut buffer = [0u8; MAX_MESSAGE_LEN];
        handshake_xx(self, &mut conn, &mut buffer).await.unwrap();
        info!("handshake complete!");
        Ok(conn)
    }
}
