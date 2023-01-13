use tokio::io::{AsyncRead, AsyncWrite};
// use sha2::Digest;
use super::*;
use crate::config::base::NewConfig;
use crate::config::tls::make_server_config;
use crate::tproto::common::request::InboundRequest;
use crate::tproto::common::stream::StandardTcpStream;
use crate::tproto::trojan;
use crate::tproto::trojan::parse_trojan;
use std::io::{Error, ErrorKind, Result};
use tokio_rustls::TlsAcceptor;

/// Acceptor handles incomming connection by escalating them to application level data stream based on
/// the configuration. It is also responsible for escalating TCP connection to TLS connection if the user
/// enabled TLS.
#[derive(Clone)]
pub struct TcpAcceptor {
    tls_acceptor: Option<TlsAcceptor>,
    port: u16,
    user_holder: UserHolderArc,
    secret: Vec<u8>, // hex
}

impl TcpAcceptor {
    pub fn init(cfg: &NewConfig, user_holder: UserHolderArc) -> Self {
        let tls_acceptor = match &cfg.tls {
            Some(tls) => match make_server_config(&tls) {
                Some(cfg) => Some(TlsAcceptor::from(cfg)),
                None => None,
            },
            None => None,
        };

        Self {
            tls_acceptor,
            port: cfg.port,
            // secret: b"xl87654321".to_vec(),
            secret: secret_to_passeord_temp(b"xl87654321"),
            user_holder,
        }
    }
    /// Takes an inbound TCP stream, escalate to TLS if possible and then escalate to application level data stream
    /// to be ready to read user's request and process them.
    pub async fn accept<T: AsyncRead + AsyncWrite + Send + Unpin>(
        &self,
        inbound_stream: T,
    ) -> Result<(InboundRequest, StandardTcpStream<T>)> {
        match &self.tls_acceptor {
            None => Ok(trojan_accept(
                StandardTcpStream::Plain(inbound_stream),
                &self.secret,
                self.user_holder.clone(),
            )
            .await?),
            Some(tls_acceptor) => {
                let tls_stream = tls_acceptor.accept(inbound_stream).await?;
                let res = trojan_accept(
                    StandardTcpStream::RustlsServer(tls_stream),
                    &self.secret,
                    self.user_holder.clone(),
                )
                .await?;
                Ok(res)
            }
        }
    }
}

/// Helper function to accept an abstract TCP stream to Trojan connection
pub async fn trojan_accept<T: AsyncRead + AsyncWrite + Unpin + Send>(
    mut stream: StandardTcpStream<T>,
    secret: &[u8],
    user_holder: UserHolderArc,
) -> Result<(InboundRequest, StandardTcpStream<T>)> {
    // Read trojan request header and generate request header
    let request = parse_trojan(&mut stream).await?;

    // Validate the request secret and decide if the connection should be accepted
    if !request.validate(secret) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Received invalid hex value",
        ));
    }

    Ok((request.into_request(), stream))
}

/////////////////// hashes ///////////

pub fn secret_to_passeord_temp(sec: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha224};

    let hash = Sha224::digest(sec)
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>()
        .as_bytes()
        .to_vec();

    hash
}

pub fn secret_to_passeord(sec: &str) -> String {
    use sha2::{Digest, Sha224};

    let hash = Sha224::digest(sec.as_bytes())
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>()
        .as_bytes()
        .to_vec();

    "st".to_string()
}
