use crate::tproto::common::addr::{IpAddrPort, IpAddress};
use crate::tproto::common::atype::Atype;
use crate::tproto::common::request::InboundRequest;
use crate::tproto::trojan::base::CRLF;
use crate::tproto::trojan::parser::parse_udp;

use log::debug;
use std::io::{self, Cursor, Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;

/// Define the size of the buffer used to transport the data back and forth
const BUF_SIZE: usize = 4096;

/// According the official documentation for Trojan protocol, the UDP data will be segmented into Trojan UDP packets,
/// which allows the outbound handler to also forward them as real UDP packets to the desired destinations.
/// Link: https://trojan-gfw.github.io/trojan/protocol.html
pub struct TrojanUdpPacketHeader {
    pub atype: Atype,
    pub dest: SocketAddr,
    pub payload_size: usize,
}

pub async fn copy_client_reader_to_udp_socket<R: AsyncRead + Unpin>(
    mut client_reader: R,
    server_writer: &UdpSocket,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];

    loop {
        let header = parse_udp(&mut client_reader).await?;

        debug!(
            "Forwarding {} bytes to {}",
            header.payload_size, header.dest
        );

        assert!(
            header.payload_size <= BUF_SIZE,
            "Payload size exceeds read buffer size"
        );

        let size = client_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        assert!(
            size == header.payload_size,
            "Failed to read the entire trojan udp packet, expect: {} bytes, read: {} bytes",
            header.payload_size,
            size
        );

        server_writer
            .send_to(&read_buf[..header.payload_size], header.dest)
            .await?;
    }
}

pub async fn copy_udp_socket_to_client_writer<W: AsyncWrite + Unpin>(
    server_reader: &UdpSocket,
    mut client_writer: W,
    addr: IpAddrPort,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let (addr, port) = (addr.ip, addr.port);

    loop {
        let (size, _dest) = server_reader.recv_from(&mut read_buf).await?;

        match addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                client_writer.write_u8(Atype::IPv4 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                client_writer.write_u8(Atype::IPv6 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                client_writer.write_u8(Atype::DomainName as u8).await?;
                client_writer
                    .write_u8(domain.as_bytes().len() as u8)
                    .await?;
                client_writer.write_all(domain.as_bytes()).await?;
            }
        }

        client_writer.write_u16(port).await?;
        client_writer.write_u16(size as u16).await?;
        client_writer.write_u16(CRLF).await?;
        client_writer.write_all(&read_buf[..size]).await?;
        client_writer.flush().await?;
    }
}
