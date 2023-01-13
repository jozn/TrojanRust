use crate::config::base::NewConfig;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;

use log::{info, warn};
use std::io::Result;

pub async fn start(cfg: &NewConfig) -> Result<()> {
    // Extract the inbound client address
    let address = (cfg.address.clone(), cfg.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Start the TCP server listener socket
    let listener = TcpListener::bind(address).await?;

    // Create TCP server acceptor and handler
    // let (acceptor, handler) = (
    //     TcpAcceptor::init(&inbound_config),
    //     TcpHandler::init(&outbound_config),
    // );

    Ok(())
}
