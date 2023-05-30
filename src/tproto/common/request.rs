use crate::tproto::common::addr::IpAddrPort;
use crate::tproto::common::addr::IpAddress;
use crate::tproto::common::atype::Atype;
use crate::tproto::common::command::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum InboundProtocol {
    TCP,
    UDP,
}

pub struct InboundRequest {
    pub atype: Atype,
    pub addr_port: IpAddrPort,
    pub command: Command,
    pub transport_protocol: InboundProtocol,
}

impl InboundRequest {
    #[inline]
    pub fn new(
        atype: Atype,
        addr: IpAddress,
        command: Command,
        port: u16,
        transport_protocol: InboundProtocol,
    ) -> Self {
        Self {
            atype,
            addr_port: IpAddrPort::new(addr, port),
            command,
            transport_protocol,
        }
    }
}
