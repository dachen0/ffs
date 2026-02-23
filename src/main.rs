use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use clap::{Parser, ValueEnum};
use ed25519_dalek::{SigningKey, VerifyingKey, pkcs8::{DecodePrivateKey, DecodePublicKey}};

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

    /// File path to a .pem file.
    /// For server mode, this is a private key.
    /// For client mode, this should be a public key corresponding
    /// to the private key that the sender is using to sign.
    #[arg(short, long)]
    key_file_path: String,

    /// Recipients
    #[arg(short, long)]
    recipients: Option<Vec<String>>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let net_config = NetConfig::new(
        IpAddr::from_str(&args.ip).expect(&("Couldn't parse provided ip ".to_owned() + &args.ip)),
        args.udp_port,
        1232,
    );

    match args.mode {
        Mode::Server => {
            let mut server = Server::new(&args.file_path, net_config).unwrap();
            let signing_key = SigningKey::read_pkcs8_pem_file(args.key_file_path).unwrap();
            server
                .send_file_to_addresses(
                    &args
                        .recipients
                        .unwrap()
                        .iter()
                        .map(|r| SocketAddr::from_str(r).unwrap())
                        .collect::<Vec<SocketAddr>>(),
                        &signing_key
                )
                .unwrap();
        }
        Mode::Client => {
            let mut client = Client::new(&args.file_path, net_config).unwrap();
            let sender_public_key = VerifyingKey::read_public_key_pem_file(args.key_file_path).unwrap();
            client.receive_file(&sender_public_key).unwrap();
        }
    };
}
