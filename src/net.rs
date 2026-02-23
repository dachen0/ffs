use std::net::{IpAddr, UdpSocket};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey, ed25519::SignatureBytes};
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
pub(crate) struct Packet {
    pub signature: Option<SignatureBytes>,
    pub packet: PacketEnum
}

// @TODO - the current Packet impl involves a lot of wincode serialize/deserialize
// to create and verify the signature.
// Instead of this we should zero copy the `PacketEnum` struct and use the ref.
impl Packet {
    pub fn new_signed_packet(packet: PacketEnum, signer: &SigningKey) -> Self {
        let mut packet = Self::new_unsigned_packet(packet);
        let signature = signer.sign(&wincode::serialize(&packet.packet).unwrap());
        packet.signature = Some(signature.to_bytes());

        packet
    }

    pub fn new_unsigned_packet(packet: PacketEnum) -> Self {
        Self {
            signature: None,
            packet
        }
    }

    /// Returns true ONLY if the signature is correct
    pub fn verify_packet(&self, signer_pubkey: &VerifyingKey) -> bool {
        if let Some(signature) = self.signature {
            return signer_pubkey.verify(&wincode::serialize(&self.packet).unwrap(), &Signature::from_bytes(&signature)).is_ok()
        }
        return false
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        wincode::serialize(&self).unwrap()
    }
}

#[derive(SchemaWrite, SchemaRead, Debug)]
pub(crate) enum PacketEnum {
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

    pub fn get_checksum(&self) -> [u8; 32] {
        *blake3::hash(&self.data).as_bytes()
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
    pub file_hash: [u8; 32],
    // list of tuples (offset, hash)
    pub checksums: Vec<(u64, [u8; 32])>,
}

impl DataMetadataPacket {
    pub fn new_from_data_packets(file_hash: [u8; 32], packets: &[DataPacket]) -> Self {
        let checksums = packets
            .iter()
            .map(|packet| (packet.offset, packet.get_checksum()))
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
