use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    net::UdpSocket,
};

use ed25519_dalek::VerifyingKey;
use log::{debug, info, warn};
use wincode::Deserialize;

use crate::net::{NetConfig, Packet, PacketEnum};

pub(crate) struct Client {
    file: File,
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
        Ok(Self { file, udp_socket })
    }

    pub fn receive_file(&mut self, sender_pubkey: &VerifyingKey) -> io::Result<()> {
        let mut file_len = u64::MAX;
        let mut bytes_written = 0;
        let mut buf = [0u8; 5192];
        let mut data_packet_checksums = BTreeMap::new();
        let mut pending_data_packets = Vec::new();
        let file = &mut self.file;
        loop {
            let (received_bytes, _src) = self.udp_socket.recv_from(&mut buf)?;
            if let Ok(packet) = Packet::deserialize(&buf[0..received_bytes]) {
                debug!("Received packet {:?}", packet);
                let packet_is_verified = packet.verify_packet(sender_pubkey);
                if packet.signature.is_some() && !packet_is_verified {
                    warn!("Packet failed verification: {:?}", packet);
                }
                match packet.packet {
                    PacketEnum::DataMetadata(metadata_packet) => {
                        if packet_is_verified {
                            for (offset, checksum) in metadata_packet.checksums {
                                data_packet_checksums.insert(offset, checksum);
                            }
                        }
                    }
                    PacketEnum::FileMetadata(file_metadata_packet) => {
                        if packet_is_verified {
                            file_len = file_metadata_packet.file_len;
                        }
                    }
                    PacketEnum::Data(data_packet) => {
                        // only accept packets that should be inside the file
                        // this before we check the checksums
                        if data_packet.offset < file_len {
                            pending_data_packets.push(data_packet);
                        }
                    }
                    _ => {}
                }
            }

            // @TODO - this triggers a vec realloc.
            // It would be best to do it in place and filter `pending_data_packets` in place
            // but it works for now.
            let currently_pending_data_packets =
                std::mem::replace(&mut pending_data_packets, Vec::new());
            for data_packet in currently_pending_data_packets {
                if let Some(checksum) = data_packet_checksums.get(&data_packet.offset) {
                    // only write the packet if matches our checksum
                    if checksum == &data_packet.get_checksum() {
                        file.seek(io::SeekFrom::Start(data_packet.offset))?;
                        file.write_all(&data_packet.data)?;
                        bytes_written += data_packet.data.len();
                    }
                } else {
                    // if we don't have the checksum for this packet yet, store it and wait for checksums
                    pending_data_packets.push(data_packet);
                }
            }

            if bytes_written as u64 == file_len {
                info!("Download of {} bytes complete", bytes_written);
                break;
            }
        }

        Ok(())
    }
}
