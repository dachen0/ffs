use std::{
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    net::UdpSocket,
};

use wincode::Deserialize;

use crate::net::{NetConfig, Packet};

pub(crate) struct Client {
    file: File,
    net_config: NetConfig,
    udp_socket: UdpSocket,
}

impl Client {
    pub fn new(file_path: &str, net_config: NetConfig) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;
        let udp_socket = net_config.get_udp_socket()?;
        Ok(Self {
            file,
            net_config,
            udp_socket,
        })
    }

    pub fn receive_file(&mut self) -> io::Result<()> {
        let mut file_len = u64::MAX;
        let mut bytes_written = 0;
        let mut buf = [0u8; 5192];
        let file = &mut self.file;
        loop {
            let (received_bytes, src) = self.udp_socket.recv_from(&mut buf)?;
            if let Ok(packet) = Packet::deserialize(&buf[0..received_bytes]) {
                match packet {
                    Packet::DataMetadata(metadata_packet) => {
                        // currently usused
                    }
                    Packet::FileMetadata(file_metadata_packet) => {
                        file_len = file_metadata_packet.file_len;
                    }
                    Packet::Data(data_packet) => {
                        file.seek(io::SeekFrom::Start(data_packet.offset))?;
                        file.write_all(&data_packet.data)?;
                        bytes_written += data_packet.data.len();
                    }
                    _ => {}
                }
            }

            if bytes_written as u64 == file_len {
                println!("download of {} bytes complete", bytes_written);
                break;
            }
        }

        Ok(())
    }
}
