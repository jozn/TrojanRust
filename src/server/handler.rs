use log::info;
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
// use tokio::io::{AsyncRead, AsyncWrite};
use crate::tproto::trojan;
use crate::tproto::trojan::*;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpStream, UdpSocket};

use crate::tproto::common::request::{InboundProtocol, InboundRequest};
use crate::tproto::common::stream::StandardTcpStream;

pub async fn handle_direct_stream<T: AsyncRead + AsyncWrite + Unpin + Send>(
    request: InboundRequest,
    inbound_stream: StandardTcpStream<T>,
) -> std::io::Result<()> {
    match request.transport_protocol {
        InboundProtocol::TCP => {
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

            let (mut client_reader, mut client_writer) = tokio::io::split(inbound_stream);
            let (mut server_reader, mut server_writer) = tokio::io::split(outbound_stream);

            // Obtain reader and writer for inbound and outbound streams
            tokio::select!(
                _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                _ = tokio::io::copy(&mut server_reader, &mut client_writer) => ()
            );
        }
        InboundProtocol::UDP => {
            // Establish UDP connection to remote host
            let socket = UdpSocket::bind("0.0.0.0:0").await?;

            let (client_reader, client_writer) = tokio::io::split(inbound_stream);

            tokio::select!(
                _ = trojan::packet::copy_client_reader_to_udp_socket(BufReader::new(client_reader), &socket) => (),
                _ = trojan::packet::copy_udp_socket_to_client_writer(&socket, BufWriter::new(client_writer), request.addr_port) => ()
            );
        }
    };

    info!("Connection finished");

    Ok(())
}
