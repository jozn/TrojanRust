use crate::proxy::base::SupportedProtocols;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewConfig {
    pub address: String,
    pub port: u16,
    pub secret: Option<String>,
    pub tls: Option<InboundTlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundTlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/////////////////////////////////////// All deprecated /////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound_dep: OutboundConfig_Dep,
}

/// Inbound traffic supports the following 3 modes: 
/// 
/// TCP - Raw TCP byte stream traffic
/// QUIC - Application level byte stream that is built on top of QUIC protocol
/// 
/// TCP and QUIC are both byte streams from the abstractions of the low level implementation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InboundMode_Dep {
    TCP,
    QUIC,
}

/// Outbound traffic supports 4 types of proxy modes:
/// 
/// DIRECT: Directly send the data in the proxy request to the requested destination, either via raw TCP or UDP
/// TCP: Forward the proxy traffic to a remote proxy server via raw TCP stream and have it take care of the traffic handling
/// QUIC: Forward the proxy traffic to a remote proxy server via QUIC stream
#[derive(Serialize, Deserialize, Clone)]
pub enum OutboundMode_Dep {
    DIRECT,
    TCP,
    QUIC,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundConfig {
    pub mode_dep: InboundMode_Dep,
    pub protocol_dep: SupportedProtocols,
    pub address: String,
    pub port: u16,
    pub secret: Option<String>,
    pub tls: Option<InboundTlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundConfig_Dep {
    pub mode: OutboundMode_Dep,
    pub protocol: SupportedProtocols,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub secret: Option<String>,
    pub tls: Option<OutboundTlsConfig_dep>,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundTlsConfig_dep {
    pub host_name: String,
    pub allow_insecure: bool,
}
