use crate::config::base::NewConfig;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;

use super::*;
use crate::server::acceptor::TcpAcceptor;
use crate::server::handler::handle_direct_stream;
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
    let user_holder = UserHolder {
        mp: Default::default(),
    };
    let user_holder_arch = Arc::new(user_holder);
    let acceptor = TcpAcceptor::init(cfg, user_holder_arch);

    // Enter server listener socket accept loop
    loop {
        info!("Ready to accept new socket connection");

        let (socket, addr) = listener.accept().await?;

        info!("Received new connection from {}", addr);

        let acceptor = acceptor.clone();
        // let (acceptor, handler) = (acceptor, handler);

        tokio::spawn(async move {
            let (request, inbound_stream) = match acceptor.accept(socket).await {
                Ok(stream) => stream,
                Err(e) => {
                    warn!("Failed to accept inbound connection from {}: {}", addr, e);
                    return;
                }
            };

            match handle_direct_stream(request, inbound_stream).await {
                Ok(_) => {
                    info!("Connection from {} has finished", addr);
                }
                Err(e) => {
                    warn!("Failed to handle the inbound stream: {}", e);
                }
            }
        });
    }
    Ok(())
}
