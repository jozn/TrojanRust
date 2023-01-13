use crate::protocol::common::addr::IpAddrPort;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::{protocol::common::addr::IpAddress, proxy::base::SupportedProtocols_Dep};

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum InboundProtocol {
    TCP,
    UDP,
}

pub struct InbounndRequest {
    pub atype: Atype,
    pub addr_port: IpAddrPort,
    pub command: Command,
    pub transport_protocol: InboundProtocol,
    pub proxy_protocol_dep: SupportedProtocols_Dep,
}

impl InbounndRequest {
    #[inline]
    pub fn new(
        atype: Atype,
        addr: IpAddress,
        command: Command,
        port: u16,
        transport_protocol: InboundProtocol,
        proxy_protocol: SupportedProtocols_Dep,
    ) -> Self {
        Self {
            atype,
            addr_port: IpAddrPort::new(addr, port),
            command,
            transport_protocol,
            proxy_protocol_dep: proxy_protocol,
        }
    }
}
