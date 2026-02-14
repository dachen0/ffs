use std::{hash::Hash, net::{IpAddr, UdpSocket}};

use wincode::{SchemaRead, SchemaWrite};

pub(crate) struct NetConfig {
    bind_ip: IpAddr,
    udp_port: u16,
    pub udp_packet_data_size: usize,
}

impl NetConfig {
    pub fn new(bind_ip: IpAddr, udp_port: u16, udp_packet_data_size: usize) -> Self {
        Self {
            bind_ip,
            udp_port,
            udp_packet_data_size
        }
    }

    pub fn get_udp_socket(&self) -> std::io::Result<UdpSocket> {
        UdpSocket::bind((self.bind_ip, self.udp_port))
    }
}

#[derive(SchemaWrite, SchemaRead)]
pub(crate) struct Packet {
    id: u64,
    pub inner: PacketType,
}

#[derive(SchemaWrite, SchemaRead)]
pub(crate) enum PacketType {
    Data(DataPacket),
    Metadata(MetadataPacket),
    Protocol(ProtocolPacket),
}

#[derive(SchemaWrite, SchemaRead, Clone)]
pub(crate) struct DataPacket {
    offset: usize,
    data: Vec<u8>
}

impl DataPacket {
    pub fn new(offset: usize, data: &[u8]) -> Self {
        // Note: this is horribly inefficient right now
        let data = data.to_vec();
        Self {
            offset,
            data
        }
    }
}

#[derive(SchemaWrite, SchemaRead)]
pub(crate) struct MetadataPacket {
    file_hash: [u8; 32],
    // list of tuples (offset, hash)
    checksums: Vec<(usize, [u8; 32])>,
}

impl MetadataPacket {
    pub fn new_from_data_packets(file_hash: [u8;32], packets: &[DataPacket]) -> Self {
        let checksums = packets.iter().map(|packet| {
            (packet.offset, *blake3::hash(&packet.data).as_bytes())
        }).collect();

        Self {
            file_hash,
            checksums
        }
    }
}

#[derive(SchemaWrite, SchemaRead)]
pub(crate) enum ProtocolPacket {
    FileRequest,
    Ack
}


#[cfg(test)]
mod tests {
    use crate::net::{DataPacket, MetadataPacket};

    #[test]
    fn sanity_metadata() {
        let data = vec![0; 100];
        let data_packets = vec![DataPacket::new(0, &data); 20];
        let _metadata_packet = MetadataPacket::new_from_data_packets([0u8; 32], &data_packets);
    }
}