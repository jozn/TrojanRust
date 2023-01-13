use crate::tproto::common::addr::IpAddress;
use crate::tproto::common::atype::Atype;
use crate::tproto::common::command::Command;
use crate::tproto::common::request::{InboundProtocol, InboundRequest};

use constant_time_eq::constant_time_eq;

use std::fmt;

/// Trojan hex payload is always 56 bytes
pub const HEX_SIZE: usize = 56;

/// Trojan protocol uses the 0x0D0A as deliminate for packet header and payload
pub const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: Vec<u8>,
    command: Command,
    atype: Atype,
    addr: IpAddress,
    port: u16,
}

impl Request {
    pub fn new(
        hex: Vec<u8>,
        command: Command,
        atype: Atype,
        addr: IpAddress,
        port: u16,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            port,
        };
    }

    #[inline]
    pub fn into_request(self) -> InboundRequest {
        return match self.command {
            Command::Udp => InboundRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                InboundProtocol::UDP,
            ),
            _ => InboundRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                InboundProtocol::TCP,
            ),
        };
    }

    #[inline]
    pub fn validate(&self, secret: &[u8]) -> bool {
        return constant_time_eq(secret, &self.hex);
    }
}

impl fmt::Display for Request {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} {}:{}",
            self.command.to_string(),
            self.addr.to_string(),
            self.port
        )
    }
}
