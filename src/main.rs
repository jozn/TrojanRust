use clap::Arg;
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use log::info;
use std::io::Result;
use trojan_rust::config::base::*;
use trojan_rust::config::parser::*;
use trojan_rust::server::server;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let new_args: ArgMatches = Command::new("Trojan Rust")
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

    // let config_path_nwe = new_args.get_one("config").unwrap_or("./config/config.json");
    let config_path_nwe = "./config.json";

    info!("Reading trojan configuration file from {}", config_path_nwe);

    let config = read_new_config(&config_path_nwe).expect("Error parsing the config file");

    server::start(&config).await?;

    Ok(())
}
