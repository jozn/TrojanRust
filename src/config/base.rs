use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewConfig {
    pub address: String,
    pub port: u16,
    pub secret: Vec<String>,
    pub list: String,
    pub tls: Option<InboundTlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundTlsConfig {
    pub cert_path: String,
    pub key_path: String,
}
