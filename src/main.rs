use clap::Arg;
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use log::info;
use std::io::Result;
use trojan_rust::config::base::*;
use trojan_rust::config::parser::*;
// use trojan_rust::proxy::quic_del;
use trojan_rust::proxy::tcp;

lazy_static! {
    static ref ARGS: ArgMatches = Command::new("Trojan Rust")
        .version("0.7.1")
        .author("cty123")
        .about("Trojan Rust is a rust implementation of the trojan protocol to circumvent GFW")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets the config file, read ./config/config.json by default")
                .takes_value(true),
        )
        .get_matches();
    static ref CONFIG_PATH: &'static str =
        ARGS.value_of("config").unwrap_or("./config/config.json");
    static ref CONFIG: (InboundConfig, OutboundConfig_Dep) = {
        let config = read_config_dep(&CONFIG_PATH).expect("Error parsing the config file");
        (config.inbound, config.outbound_dep)
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!(
        "Reading trojan configuration file from {}",
        CONFIG_PATH.to_string()
    );

    // // TODO: Support more types of server, like UDP
    // match CONFIG.0.mode_dep {
    //     InboundMode::TCP => {
    //         // tcp::server::start(&CONFIG.0, &CONFIG.1).await?;
    //     }
    //     InboundMode::QUIC => {
    //         // quic_del::server::start(&CONFIG.0, &CONFIG.1).await?;
    //     }
    // }

    tcp::server::start(&CONFIG.0, &CONFIG.1).await?;

    Ok(())
}
