use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    net::{SocketAddr, UdpSocket},
};

use ed25519_dalek::SigningKey;
use wincode::Deserialize;

use crate::{
    fs::get_file_hash,
    net::{DataMetadataPacket, DataPacket, FileMetadataPacket, NetConfig, Packet, PacketEnum, ProtocolPacket},
};

pub(crate) struct Server {
    file: File,
    file_path: String,
    file_hash: [u8; 32],
    net_config: NetConfig,
    udp_socket: UdpSocket,
}

impl Server {
    pub fn new(file_path: &str, net_config: NetConfig) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_hash = get_file_hash(&file)?;
        let udp_socket = net_config.get_udp_socket()?;
        Ok(Self {
            file,
            file_path: file_path.into(),
            file_hash,
            net_config,
            udp_socket,
        })
    }

    pub fn _listen(&mut self, signing_key: &SigningKey) -> io::Result<()> {
        let mut buf = [0u8; 5192];
        loop {
            let (received_bytes, src) = self.udp_socket.recv_from(&mut buf)?;
            if let Ok(packet) = Packet::deserialize(&buf[0..received_bytes]) {
                match packet.packet {
                    PacketEnum::DataMetadata(_metadata_packet) => todo!(),
                    PacketEnum::Data(_data_packet) => todo!(),
                    PacketEnum::Protocol(protocol_packet) => match protocol_packet {
                        ProtocolPacket::FileRequest => {
                            self.send_file_to_addresses(&[src], signing_key)?;
                        }
                        ProtocolPacket::Ack => todo!(),
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn send_file_to_addresses(&mut self, recipients: &[SocketAddr], signing_key: &SigningKey) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut file_reader = BufReader::new(&self.file);

        // notify the clients of the file size
        let file_metadata = std::fs::metadata(&self.file_path)?;
        let file_metadata_packet = Packet::new_signed_packet(PacketEnum::FileMetadata(FileMetadataPacket::new(
            self.file_path.clone(),
            self.file_hash,
            file_metadata.len(),
        )), signing_key);
        let serialized_file_metadata_packet = wincode::serialize(&file_metadata_packet).unwrap();
        debug_assert_eq!(
            self.udp_socket
                .send_to(&serialized_file_metadata_packet.as_slice(), recipients)?,
            serialized_file_metadata_packet.len()
        );

        let mut offset: u64 = 0;
        loop {
            let buf = file_reader.fill_buf()?;
            let buf_len = buf.len();

            // EOF
            if buf.len() == 0 {
                break;
            }

            // chunk the buffer into packet sized bites
            let data_chunk_size = self.net_config.udp_packet_data_size;
            let packets: Vec<DataPacket> = buf
                .chunks(data_chunk_size as usize)
                .enumerate()
                .map(|(index, chunk)| {
                    DataPacket::new(offset + (index as u64) * data_chunk_size, chunk)
                })
                .collect();

            let metadata_packet = Packet::new_signed_packet(PacketEnum::DataMetadata(DataMetadataPacket::new_from_data_packets(
                self.file_hash,
                &packets,
            )), signing_key);
            let serialized_metadata_packet = metadata_packet.to_bytes();
            debug_assert_eq!(
                self.udp_socket
                    .send_to(serialized_metadata_packet.as_slice(), recipients)?,
                serialized_metadata_packet.len()
            );

            for packet in packets {
                // TODO: Store buffers that we can write into
                let serialized_packet = Packet::new_unsigned_packet(PacketEnum::Data(packet)).to_bytes();
                debug_assert_eq!(
                    self.udp_socket
                        .send_to(serialized_packet.as_slice(), recipients)?,
                    serialized_packet.len()
                );
            }

            offset += buf_len as u64;
            file_reader.consume(buf_len);
        }

        return Ok(());
    }
}
