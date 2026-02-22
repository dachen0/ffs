use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use clap::{Parser, ValueEnum};

use crate::{client::Client, net::NetConfig, server::Server};

mod client;
mod fs;
mod net;
mod server;

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

    /// Recipients
    #[arg(short, long)]
    recipients: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();

    let net_config = NetConfig::new(
        IpAddr::from_str(&args.ip).expect(&("Couldn't parse provided ip ".to_owned() + &args.ip)),
        args.udp_port,
        1232,
    );

    match args.mode {
        Mode::Server => {
            let mut server = Server::new(&args.file_path, net_config).unwrap();
            server
                .send_file_to_addresses(
                    &args
                        .recipients
                        .unwrap()
                        .iter()
                        .map(|r| SocketAddr::from_str(r).unwrap())
                        .collect::<Vec<SocketAddr>>(),
                )
                .unwrap();
        }
        Mode::Client => {
            let mut client = Client::new(&args.file_path, net_config).unwrap();
            client.receive_file().unwrap();
        }
    };
}
