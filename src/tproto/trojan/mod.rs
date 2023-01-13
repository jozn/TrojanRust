mod base;
mod parser;

pub mod packet;

pub use self::base::CRLF;
pub use self::base::HEX_SIZE;
pub use self::parser::parse_trojan;
//
// use crate::tproto::common::addr::IpAddress;
// use crate::tproto::common::{request::InboundRequest, stream::StandardTcpStream};
//
// use std::io::{Error, ErrorKind, Result};
// use std::net::IpAddr;
// use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
//
// /// Helper function to accept an abstract TCP stream to Trojan connection
// pub async fn accept_dep<T: AsyncRead + AsyncWrite + Unpin + Send>(
//     mut stream: StandardTcpStream<T>,
//     secret: &[u8],
//     user_holder: &[u8],
// ) -> Result<(InboundRequest, StandardTcpStream<T>)> {
//     // Read trojan request header and generate request header
//     let request = parse_trojan(&mut stream).await?;
//
//     // Validate the request secret and decide if the connection should be accepted
//     if !request.validate(secret) {
//         return Err(Error::new(
//             ErrorKind::InvalidInput,
//             "Received invalid hex value",
//         ));
//     }
//
//     Ok((request.into_request(), stream))
// }
