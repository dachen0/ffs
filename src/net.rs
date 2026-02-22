use std::{
    hash::Hash,
    net::{IpAddr, UdpSocket},
};

use wincode::{SchemaRead, SchemaWrite};

pub(crate) struct NetConfig {
    bind_ip: IpAddr,
    udp_port: u16,
    pub udp_packet_data_size: u64,
}

impl NetConfig {
    pub fn new(bind_ip: IpAddr, udp_port: u16, udp_packet_data_size: u64) -> Self {
        Self {
            bind_ip,
            udp_port,
            udp_packet_data_size,
        }
    }

    pub fn get_udp_socket(&self) -> std::io::Result<UdpSocket> {
        UdpSocket::bind((self.bind_ip, self.udp_port))
    }
}

#[derive(SchemaWrite, SchemaRead, Debug)]
pub(crate) enum Packet {
    Data(DataPacket),
    FileMetadata(FileMetadataPacket),
    DataMetadata(DataMetadataPacket),
    Protocol(ProtocolPacket),
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub(crate) struct DataPacket {
    pub offset: u64,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(offset: u64, data: &[u8]) -> Self {
        // Note: this is horribly inefficient right now
        let data = data.to_vec();
        Self { offset, data }
    }
}

#[derive(SchemaWrite, SchemaRead, Debug)]
pub(crate) struct FileMetadataPacket {
    pub file_path: String,
    pub file_hash: [u8; 32],
    pub file_len: u64,
}

impl FileMetadataPacket {
    pub fn new(file_path: String, file_hash: [u8; 32], len: u64) -> Self {
        Self {
            file_path,
            file_hash,
            file_len: len,
        }
    }
}

#[derive(SchemaWrite, SchemaRead, Debug)]
pub(crate) struct DataMetadataPacket {
    file_hash: [u8; 32],
    // list of tuples (offset, hash)
    checksums: Vec<(u64, [u8; 32])>,
}

impl DataMetadataPacket {
    pub fn new_from_data_packets(file_hash: [u8; 32], packets: &[DataPacket]) -> Self {
        let checksums = packets
            .iter()
            .map(|packet| (packet.offset, *blake3::hash(&packet.data).as_bytes()))
            .collect();

        Self {
            file_hash,
            checksums,
        }
    }
}

#[derive(SchemaWrite, SchemaRead, Debug)]
pub(crate) enum ProtocolPacket {
    FileRequest,
    Ack,
}

#[cfg(test)]
mod tests {
    use crate::net::{DataMetadataPacket, DataPacket};

    #[test]
    fn sanity_metadata() {
        let data = vec![0; 100];
        let data_packets = vec![DataPacket::new(0, &data); 20];
        let _metadata_packet = DataMetadataPacket::new_from_data_packets([0u8; 32], &data_packets);
    }
}
