use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc32fast::Hasher;
use std::io::{Cursor, Read, Write};

pub const MAGIC_NUMBER: u32 = 0x51524654; // "QRFT"

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Metadata(MetadataFrame),
    Data(DataFrame),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFrame {
    pub version: u8,
    pub transfer_id: u32,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub compression_type: u8, // 0 = None, 1 = Zstd
    pub total_symbols: u32,
    pub symbol_size: u16,
    pub sha256_hash: [u8; 32],
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
    pub transfer_id: u32,
    pub encoding_symbol_id: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_u32::<BigEndian>(MAGIC_NUMBER).unwrap();

        match self {
            Frame::Metadata(m) => {
                buf.write_u8(0x00).unwrap();
                buf.write_u8(m.version).unwrap();
                buf.write_u32::<BigEndian>(m.transfer_id).unwrap();
                buf.write_u64::<BigEndian>(m.uncompressed_size).unwrap();
                buf.write_u64::<BigEndian>(m.compressed_size).unwrap();
                buf.write_u8(m.compression_type).unwrap();
                buf.write_u32::<BigEndian>(m.total_symbols).unwrap();
                buf.write_u16::<BigEndian>(m.symbol_size).unwrap();
                buf.write_all(&m.sha256_hash).unwrap();
                buf.write_u8(m.filename.len() as u8).unwrap();
                buf.write_all(m.filename.as_bytes()).unwrap();
            }
            Frame::Data(d) => {
                buf.write_u8(0x01).unwrap();
                buf.write_u32::<BigEndian>(d.transfer_id).unwrap();
                buf.write_u32::<BigEndian>(d.encoding_symbol_id).unwrap();
                buf.write_u16::<BigEndian>(d.payload.len() as u16).unwrap();
                buf.write_all(&d.payload).unwrap();
            }
        }

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc = hasher.finalize();
        buf.write_u32::<BigEndian>(crc).unwrap();
        
        buf
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }

        let mut cursor = Cursor::new(data);
        let magic = cursor.read_u32::<BigEndian>().ok()?;
        if magic != MAGIC_NUMBER {
            return None;
        }

        // Verify CRC
        let mut hasher = Hasher::new();
        hasher.update(&data[..data.len() - 4]);
        let expected_crc = hasher.finalize();
        
        let mut crc_cursor = Cursor::new(&data[data.len() - 4..]);
        let actual_crc = crc_cursor.read_u32::<BigEndian>().ok()?;
        if expected_crc != actual_crc {
            return None;
        }

        let frame_type = cursor.read_u8().ok()?;
        match frame_type {
            0x00 => {
                let version = cursor.read_u8().ok()?;
                let transfer_id = cursor.read_u32::<BigEndian>().ok()?;
                let uncompressed_size = cursor.read_u64::<BigEndian>().ok()?;
                let compressed_size = cursor.read_u64::<BigEndian>().ok()?;
                let compression_type = cursor.read_u8().ok()?;
                let total_symbols = cursor.read_u32::<BigEndian>().ok()?;
                let symbol_size = cursor.read_u16::<BigEndian>().ok()?;
                let mut sha256_hash = [0u8; 32];
                cursor.read_exact(&mut sha256_hash).ok()?;
                let filename_len = cursor.read_u8().ok()?;
                let mut filename_bytes = vec![0u8; filename_len as usize];
                cursor.read_exact(&mut filename_bytes).ok()?;
                let filename = String::from_utf8(filename_bytes).ok()?;

                Some(Frame::Metadata(MetadataFrame {
                    version,
                    transfer_id,
                    uncompressed_size,
                    compressed_size,
                    compression_type,
                    total_symbols,
                    symbol_size,
                    sha256_hash,
                    filename,
                }))
            }
            0x01 => {
                let transfer_id = cursor.read_u32::<BigEndian>().ok()?;
                let encoding_symbol_id = cursor.read_u32::<BigEndian>().ok()?;
                let payload_len = cursor.read_u16::<BigEndian>().ok()?;
                let mut payload = vec![0u8; payload_len as usize];
                cursor.read_exact(&mut payload).ok()?;
                
                Some(Frame::Data(DataFrame {
                    transfer_id,
                    encoding_symbol_id,
                    payload,
                }))
            }
            _ => None,
        }
    }
}
