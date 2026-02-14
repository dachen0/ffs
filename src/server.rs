use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Seek, SeekFrom}, net::{SocketAddr, UdpSocket},
};

use wincode::Deserialize;

use crate::{fs::get_file_hash, net::{DataPacket, MetadataPacket, NetConfig, Packet, PacketType, ProtocolPacket}};

pub(crate) struct Server {
    file: File,
    file_hash: [u8; 32],
    net_config: NetConfig,
    udp_socket: UdpSocket,
}

impl Server {
    pub fn new(file_path: &str, net_config: NetConfig) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_hash = get_file_hash(&file)?;
        let udp_socket = net_config.get_udp_socket()?;
        Ok(Self { file, file_hash, net_config, udp_socket })
    }

    pub fn listen(&mut self) -> io::Result<()> {
        let mut buf = [0u8; 5192];
        loop {
            let (received_bytes, src) = self.udp_socket.recv_from(&mut buf)?;
            if let Ok(packet) = Packet::deserialize(&buf[0..received_bytes]) {
                match packet.inner {
                    PacketType::Metadata(metadata_packet) => todo!(),
                    PacketType::Data(data_packet) => todo!(),
                    PacketType::Protocol(protocol_packet) => {
                        match protocol_packet {
                            ProtocolPacket::FileRequest => {
                                self.send_file_to_addresses(&[src])?;
                            },
                            ProtocolPacket::Ack => todo!(),
                        }
                    }
                }
            }
        }
    }

    pub fn send_file_to_addresses(&mut self, recipients: &[SocketAddr]) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut file_reader = BufReader::new(&self.file);
        
        let mut offset: usize = 0;
        loop {
            let buf = file_reader.fill_buf()?;
            
            // EOF
            if buf.len() == 0 {
                break;
            }

            // chunk the buffer into packet sized bites
            let data_chunk_size = self.net_config.udp_packet_data_size;
            let packets: Vec<DataPacket> = buf.chunks(data_chunk_size)
                .enumerate()
                .map(|(index, chunk)| {
                    DataPacket::new(offset + index * data_chunk_size, chunk)
                }).collect();
            
            let metadata_packet = MetadataPacket::new_from_data_packets(self.file_hash, &packets);
            let serialized_metadata_packet = wincode::serialize(&metadata_packet).unwrap();
            self.udp_socket.send_to(serialized_metadata_packet.as_slice(), recipients)?;

            for packet in packets {
                // TODO: Store buffers that we can write into
                let serialized_packet = wincode::serialize(&packet).unwrap();
                self.udp_socket.send_to(serialized_packet.as_slice(), recipients)?;
            }

            offset += buf.len();
        }

        return Ok(())
    }
}
