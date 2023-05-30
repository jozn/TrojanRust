use log::error;

use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::sync::Arc;

use crate::config::base::InboundTlsConfig;
use rustls::Error;
use rustls::RootCertStore;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{read_one, Item};

/// Create ServerConfig for rustls based on the configurations in the config.json file. The function
pub fn make_server_config(config: &InboundTlsConfig) -> Option<Arc<ServerConfig>> {
    let certificates = match load_certs(&config.cert_path) {
        Ok(certs) => certs,
        Err(_) => return None,
    };

    let key = match load_private_key(&config.key_path) {
        Ok(key) => key,
        Err(_) => return None,
    };

    let cfg = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .expect("bad certificate/key");

    Some(Arc::new(cfg))
}

fn load_certs(path: &str) -> std::io::Result<Vec<Certificate>> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    return match rustls_pemfile::certs(&mut reader) {
        Ok(certs) => Ok(certs.into_iter().map(|bytes| Certificate(bytes)).collect()),
        Err(_) => Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "failed to load tls certificate",
        )),
    };
}

fn load_private_key(path: &str) -> std::io::Result<PrivateKey> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => return Err(e),
    };

    return match read_one(&mut reader) {
        Ok(opt) => match opt {
            Some(item) => match item {
                Item::RSAKey(key) => Ok(rustls::PrivateKey(key)),
                Item::PKCS8Key(key) => Ok(rustls::PrivateKey(key)),
                Item::ECKey(key) => Ok(rustls::PrivateKey(key)),
                _ => Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Found cert in ssl key file",
                )),
            },
            None => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Failed to find any private key in file",
            )),
        },
        Err(e) => Err(e),
    };
}
