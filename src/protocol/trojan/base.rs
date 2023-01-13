use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::common::request::{InboundProtocol, InbounndRequest};
use crate::proxy::base::SupportedProtocols_Dep;

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
    proxy_protocol: SupportedProtocols_Dep,
}

impl Request {
    pub fn new(
        hex: Vec<u8>,
        command: Command,
        atype: Atype,
        addr: IpAddress,
        port: u16,
        proxy_protocol: SupportedProtocols_Dep,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            port,
            proxy_protocol,
        };
    }

    #[inline]
    pub fn into_request(self) -> InbounndRequest {
        return match self.command {
            Command::Udp => InbounndRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                InboundProtocol::UDP,
                self.proxy_protocol,
            ),
            _ => InbounndRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                InboundProtocol::TCP,
                self.proxy_protocol,
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
