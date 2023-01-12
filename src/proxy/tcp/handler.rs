use crate::config::base::{OutboundConfig_Dep, OutboundMode_Dep};
use crate::config::tls::{make_client_config, NoCertificateVerification};
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::StandardTcpStream;
use crate::protocol::trojan::{self, handshake, HEX_SIZE};
use crate::proxy::base::SupportedProtocols;

use futures::Stream;
use log::info;
use once_cell::sync::OnceCell;
use rustls::{ClientConfig, ServerName};
use sha2::{Digest, Sha224};
use std::io::{self, Cursor, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc::{self, Sender};
use tokio_rustls::TlsConnector;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
// use tonic::Status;

/// Static life time TCP server outbound traffic handler to avoid ARC
/// The handler is initialized through init() function
static TCP_HANDLER: OnceCell<TcpHandler> = OnceCell::new();

/// Handler is responsible for taking user's request and process them and send back the result.
/// It may need to dial to remote using TCP, UDP and TLS, in which it will be responsible for
/// establishing a tranport level connection and escalate it to application data stream.
pub struct TcpHandler {
    mode: OutboundMode_Dep,
    protocol: SupportedProtocols,
    destination: Option<SocketAddr>,
    tls: Option<(Arc<ClientConfig>, ServerName)>,
    secret: Vec<u8>,
}

impl TcpHandler {
    /// Instantiate a new Handler instance based on OutboundConfig passed by the user. It will evaluate the
    /// TLS option particularly to be able to later determine whether it should escalate the connection to
    /// TLS first or not.
    pub fn init(outbound: &OutboundConfig_Dep) -> &'static TcpHandler {
        // Get outbound TLS configuration and host dns name if TLS is enabled
        let tls = match &outbound.tls {
            Some(cfg) => {
                let client_config = make_client_config(&cfg);
                Some((
                    client_config,
                    ServerName::try_from(cfg.host_name.as_ref())
                        .expect("Failed to parse host name"),
                ))
            }
            None => None,
        };

        // Attempt to extract destination address and port from OutboundConfig.
        let destination = match (outbound.address.clone(), outbound.port) {
            (Some(addr), Some(port)) => match format!("{}:{}", addr, port).parse() {
                Ok(s) => Some(s),
                Err(e) => panic!("Failed to parse destination address: {}", e),
            },
            (Some(_), None) => {
                panic!("Missing port while address is present")
            }
            (None, Some(_)) => {
                panic!("Missing address while address is present")
            }
            // No destination address and port specified, will use the address and port in each request
            (None, None) => None,
        };

        // Extract the plaintext of the secret and process it
        let secret = match outbound.protocol {
            SupportedProtocols::TROJAN if outbound.secret.is_some() => {
                let secret = outbound.secret.clone().unwrap();
                Sha224::digest(secret.as_bytes())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            }
            // Configure secret if need to add other protocols
            _ => Vec::new(),
        };

        TCP_HANDLER.get_or_init(|| Self {
            mode: outbound.mode.clone(),
            protocol: outbound.protocol,
            destination,
            tls,
            secret,
        })
    }

    /// Given an abstract inbound stream, it will read the request to standard request format and then process it.
    /// After taking the request, the handler will then establish the outbound connection based on the user configuration,
    /// and transport data back and forth until one side terminate the connection.
    #[inline]
    pub async fn dispatch<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        inbound_stream: StandardTcpStream<T>,
        request: InboundRequest,
    ) -> io::Result<()> {
        match self.mode {
            OutboundMode_Dep::DIRECT => self.handle_direct_stream(request, inbound_stream).await?,
            OutboundMode_Dep::TCP => self.handle_tcp_stream(request, inbound_stream).await?,
            _ => (),
            // OutboundMode_Dep::QUIC => self.handle_quic_stream(request, inbound_stream).await?,
        }

        Ok(())
    }

    /// Handle inbound TCP stream with direct outbound proxy strategy. Based on the inbound request, the handler
    /// will need to determine the way the input data is encrypted from the proxy request body and decrypt it to
    /// get the actual payload. Finally, it forwards the payload directly either with TCP or UDP flow.
    async fn handle_direct_stream<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardTcpStream<T>,
    ) -> io::Result<()> {
        let (proxy_protocol, transport_protocol) =
            (request.proxy_protocol, request.transport_protocol);

        // Based on the protocol in the request body, decrypt the payload respectively
        match proxy_protocol {
            SupportedProtocols::TROJAN => {
                match transport_protocol {
                    TransportProtocol::TCP => {
                        // Extract the destination port and address from the proxy request
                        let addr: SocketAddr = match request.addr_port.into() {
                            Ok(addr) => addr,
                            Err(e) => return Err(e),
                        };

                        // Connect to remote server from the proxy request
                        let outbound_stream = match TcpStream::connect(addr).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                return Err(Error::new(
                                    ErrorKind::ConnectionRefused,
                                    format!("failed to connect to tcp {}: {}", addr, e),
                                ))
                            }
                        };

                        let (mut client_reader, mut client_writer) =
                            tokio::io::split(inbound_stream);
                        let (mut server_reader, mut server_writer) =
                            tokio::io::split(outbound_stream);

                        // Obtain reader and writer for inbound and outbound streams
                        tokio::select!(
                            _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                            _ = tokio::io::copy(&mut server_reader, &mut client_writer) => ()
                        );
                    }
                    TransportProtocol::UDP => {
                        // Establish UDP connection to remote host
                        let socket = UdpSocket::bind("0.0.0.0:0").await?;

                        let (client_reader, client_writer) = tokio::io::split(inbound_stream);

                        tokio::select!(
                            _ = trojan::packet::copy_client_reader_to_udp_socket(BufReader::new(client_reader), &socket) => (),
                            _ = trojan::packet::copy_udp_socket_to_client_writer(&socket, BufWriter::new(client_writer), request.addr_port) => ()
                        );
                    }
                };
            }
            // Handler currently doesn't support SOCKS protocol.
            // Also not sure if we should support SOCKS protocol for the scope of this project.
            // SupportedProtocols::SOCKS => {
            //     return Err(Error::new(
            //         ErrorKind::Unsupported,
            //         "Proxy request can't have socks as proxy protocol",
            //     ))
            // }
            SupportedProtocols::DIRECT => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Proxy request can't have direct as proxy protocol",
                ))
            }
        };

        info!("Connection finished");

        Ok(())
    }

    /// #Experimental functionality
    /// QUIC support is currently experimental.
    // async fn handle_quic_stream<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
    //     &self,
    //     request: InboundRequest,
    //     inbound_stream: StandardTcpStream<T>,
    // ) -> io::Result<()> {
        // // Dial remote proxy server
        // let _roots = rustls::RootCertStore::empty();
        // let client_crypto = rustls::ClientConfig::builder()
        //     .with_safe_defaults()
        //     .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
        //     .with_no_client_auth();
        // let mut endpoint = quinn::Endpoint::client("[::]:0".parse().unwrap()).unwrap();
        // endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_crypto)));
        //
        // // Establish connection with remote proxy server using QUIC protocol
        // let connection = endpoint
        //     .connect("127.0.0.1:8081".parse().unwrap(), "example.com")
        //     .unwrap()
        //     .await
        //     .unwrap();
        //
        // let quinn::NewConnection {
        //     connection: conn, ..
        // } = connection;
        //
        // let (mut server_writer, mut server_reader) = conn.open_bi().await.unwrap();
        // let (mut client_reader, mut client_writer) = tokio::io::split(inbound_stream);
        //
        // handshake(&mut server_writer, &request, &self.secret).await?;
        //
        // tokio::select!(
        //     _ = tokio::spawn(async move {tokio::io::copy(&mut client_reader, &mut server_writer).await}) => (),
        //     _ = tokio::spawn(async move {tokio::io::copy(&mut server_reader, &mut client_writer).await}) => (),
        // );

        // Ok(())
    // }

    /// Handle inbound TCP stream with TCP outbound proxy strategy. This function is used when the program serves as
    /// the client end of proxy chain, such that it read the plaintext data from the inbound stream and will encrypt
    /// the it with the selected proxy and forward the proxy request to remote server.
    async fn handle_tcp_stream<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardTcpStream<T>,
    ) -> io::Result<()> {
        // Establish the initial connection with remote server
        let connection = match self.destination {
            Some(dest) => TcpStream::connect(dest).await?,
            None => {
                return Err(Error::new(
                    ErrorKind::NotConnected,
                    "missing address of the remote server",
                ))
            }
        };

        // Escalate the connection to TLS connection if tls config is present
        let mut outbound_stream = match &self.tls {
            Some((client_config, domain)) => {
                let connector = TlsConnector::from(client_config.clone());
                StandardTcpStream::RustlsClient(
                    connector.connect(domain.clone(), connection).await?,
                )
            }
            None => StandardTcpStream::Plain(connection),
        };

        // Handshake to form the proxy stream
        match self.protocol {
            SupportedProtocols::TROJAN => {
                // Check Trojan secret match
                if self.secret.len() != HEX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Hex in trojan protocol is not {} bytes", HEX_SIZE),
                    ));
                }

                // Start handshake to establish proxy stream
                handshake(&mut outbound_stream, &request, &self.secret).await?;

                match request.transport_protocol {
                    TransportProtocol::TCP => {
                        let (mut client_reader, mut client_writer) =
                            tokio::io::split(inbound_stream);
                        let (mut server_reader, mut server_writer) =
                            tokio::io::split(outbound_stream);

                        // Obtain reader and writer for inbound and outbound streams
                        tokio::select!(
                            _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                            _ = tokio::io::copy(&mut server_reader, &mut client_writer) => ()
                        );
                    }
                    TransportProtocol::UDP => {
                        let (client_reader, client_writer) = tokio::io::split(inbound_stream);
                        let (server_reader, server_writer) = tokio::io::split(outbound_stream);

                        tokio::select!(
                            _ = trojan::packet::copy_client_reader_to_udp_server_writer(client_reader, BufWriter::new(server_writer), request) => (),
                            _ = trojan::packet::copy_udp_server_reader_to_client_writer(BufReader::new(server_reader), client_writer) => (),
                        );
                    }
                }
            }
            // SupportedProtocols::SOCKS => {
            //     return Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"))
            // }
            SupportedProtocols::DIRECT => {
                return Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"));
            }
        };

        info!("Connection finished");
        Ok(())
    }
}