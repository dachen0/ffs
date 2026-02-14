use std::{net::IpAddr, str::FromStr};

use clap::{Parser, ValueEnum};

use crate::{net::NetConfig, server::Server};

mod client;
mod net;
mod server;
mod fs;

#[derive(ValueEnum, Clone, Parser, Debug)]
enum Mode {
    Server,
    Client,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Source file
    #[arg(short, long)]
    file_path: String,

    /// Mode
    #[arg(short, long)]
    mode: Mode,

    /// IP
    #[arg(short, long)]
    ip: String,

    /// Port
    #[arg(short, long)]
    udp_port: u16,
}

fn main() {
    let args = Args::parse();

    let net_config = NetConfig::new(IpAddr::from_str(&args.ip).expect(&("Couldn't parse provided ip ".to_owned() + &args.ip)), args.udp_port, 1232);

    match args.mode {
        Mode::Server => {
            let server = Server::new(&args.file_path, net_config).unwrap();
        }
        Mode::Client => {}
    };
}
